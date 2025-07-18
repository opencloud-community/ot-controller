// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::BTreeSet, time::Duration};

use either::Either;
use futures::{FutureExt, stream::once};
use opentalk_signaling_core::{
    CleanupScope, DestroyContext, Event, InitContext, ModuleContext, SignalingModule,
    SignalingModuleError, SignalingModuleInitData, SignalingRoomId, VolatileStorage, control,
};
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::Role;
use opentalk_types_signaling_polls::{
    Choice, ChoiceId, MODULE_ID, PollId, Results,
    command::{PollsCommand, Start, Vote},
    event::{Error, PollsEvent, Started},
    state::PollsState,
};
use snafu::Report;
use storage::PollsStorage;
use tokio::time::sleep;

pub mod exchange;
mod storage;

pub struct ExpiredEvent(PollId);

pub struct Polls {
    room: SignalingRoomId,
    config: Option<Config>,
}

trait PollsStorageProvider {
    fn storage(&mut self) -> &mut dyn PollsStorage;
}

impl PollsStorageProvider for VolatileStorage {
    fn storage(&mut self) -> &mut dyn PollsStorage {
        match self.as_mut() {
            Either::Left(v) => v,
            Either::Right(v) => v,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for Polls {
    const NAMESPACE: ModuleId = MODULE_ID;

    type Params = ();

    type Incoming = PollsCommand;
    type Outgoing = PollsEvent;
    type ExchangeMessage = exchange::Message;

    type ExtEvent = ExpiredEvent;

    type FrontendData = PollsState;
    type PeerFrontendData = ();

    async fn init(
        ctx: InitContext<'_, Self>,
        _: &Self::Params,
        _: &'static str,
    ) -> Result<Option<Self>, SignalingModuleError> {
        Ok(Some(Self {
            room: ctx.room_id(),
            config: None,
        }))
    }

    async fn on_event(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        event: Event<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        match event {
            Event::Joined {
                control_data: _,
                frontend_data,
                participants: _,
            } => {
                if let Some(polls_state) = ctx.volatile.storage().get_polls_state(self.room).await?
                {
                    if let Some(duration) = polls_state.remaining() {
                        let id = polls_state.id;

                        self.config = Some(Config {
                            state: polls_state.clone(),
                            voted_choice_ids: BTreeSet::new(),
                        });
                        *frontend_data = Some(polls_state);

                        ctx.add_event_stream(once(sleep(duration).map(move |_| ExpiredEvent(id))));
                    }
                }

                Ok(())
            }
            Event::Leaving => Ok(()),
            Event::RaiseHand => Ok(()),
            Event::LowerHand => Ok(()),
            Event::ParticipantJoined(_, _) => Ok(()),
            Event::ParticipantLeft(_) => Ok(()),
            Event::ParticipantUpdated(_, _) => Ok(()),
            Event::RoleUpdated(_) => Ok(()),
            Event::WsMessage(msg) => self.on_ws_message(ctx, msg).await,
            Event::Exchange(msg) => self.on_exchange_message(ctx, msg).await,
            Event::Ext(ExpiredEvent(id)) => {
                if let Some(config) = self.config.as_ref().filter(|config| config.state.id == id) {
                    let results = ctx
                        .volatile
                        .storage()
                        .poll_results(self.room, &config.state)
                        .await?;

                    ctx.ws_send(PollsEvent::Done(Results { id, results }));
                }

                Ok(())
            }
        }
    }

    async fn on_destroy(self, mut ctx: DestroyContext<'_>) {
        match ctx.cleanup_scope {
            CleanupScope::None => (),
            CleanupScope::Local => Polls::cleanup_room(&mut ctx, self.room).await,
            CleanupScope::Global => {
                if self.room.breakout_room_id().is_some() {
                    Polls::cleanup_room(&mut ctx, SignalingRoomId::new(self.room.room_id(), None))
                        .await
                }

                Polls::cleanup_room(&mut ctx, self.room).await
            }
        }
    }

    async fn build_params(
        _init: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
        Ok(Some(()))
    }
}

impl Polls {
    fn is_running(&self) -> bool {
        self.config
            .as_ref()
            .map(|config| !config.state.is_expired())
            .unwrap_or_default()
    }

    async fn on_ws_message(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        msg: PollsCommand,
    ) -> Result<(), SignalingModuleError> {
        match msg {
            PollsCommand::Start(Start {
                topic,
                live,
                multiple_choice,
                choices,
                duration,
            }) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::InsufficientPermissions);

                    return Ok(());
                }

                if self.is_running() {
                    ctx.ws_send(Error::StillRunning);

                    return Ok(());
                }

                // TODO(k.balt): Minimal duration 2 secs for tests but thats unreasonably low real world applications
                let min = Duration::from_secs(2);
                let max = Duration::from_secs(3600);

                if duration > max || duration < min {
                    ctx.ws_send(Error::InvalidDuration);

                    return Ok(());
                }

                if !matches!(topic.len(), 2..=100) {
                    ctx.ws_send(Error::InvalidTopicLength);

                    return Ok(());
                }

                if !matches!(choices.len(), 2..=64) {
                    ctx.ws_send(Error::InvalidChoiceCount);

                    return Ok(());
                }

                if choices
                    .iter()
                    .any(|content| !matches!(content.len(), 1..=100))
                {
                    ctx.ws_send(Error::InvalidChoiceDescription);

                    return Ok(());
                }

                let choices = choices
                    .into_iter()
                    .enumerate()
                    .map(|(i, content)| Choice {
                        id: ChoiceId::from(i as u32),
                        content,
                    })
                    .collect();

                let polls_state = PollsState {
                    id: PollId::generate(),
                    topic,
                    live,
                    multiple_choice,
                    choices,
                    started: ctx.timestamp(),
                    duration,
                };

                let set = ctx
                    .volatile
                    .storage()
                    .set_polls_state(self.room, &polls_state)
                    .await?;

                if !set {
                    ctx.ws_send(Error::StillRunning);

                    return Ok(());
                }

                ctx.volatile
                    .storage()
                    .add_poll_to_list(self.room, polls_state.id)
                    .await?;

                ctx.exchange_publish(
                    control::exchange::current_room_all_participants(self.room),
                    exchange::Message::Started(polls_state),
                );

                Ok(())
            }
            PollsCommand::Vote(Vote { poll_id, choices }) => {
                let Some(config) = self
                    .config
                    .as_mut()
                    .filter(|config| config.state.id == poll_id && !config.state.is_expired())
                else {
                    ctx.ws_send(Error::InvalidPollId);
                    return Ok(());
                };

                let choice_ids = choices.to_hash_set();

                if choice_ids.len() > 1 && !config.state.multiple_choice {
                    ctx.ws_send(Error::MultipleChoicesNotAllowed);
                    return Ok(());
                }

                // TODO(w.rabl) Currently the user's choices are stored in-memory only and thus they are lost after reconnecting.
                // In this case, if the user sends a new set of choices, the previous choices can't be properly reverted.
                // This leads to an inconsistent poll result.
                let valid_choice_ids: BTreeSet<ChoiceId> = config
                    .state
                    .choices
                    .iter()
                    .map(|choice| choice.id)
                    .collect();

                if choice_ids.is_subset(&valid_choice_ids) {
                    ctx.volatile
                        .storage()
                        .vote(
                            self.room,
                            config.state.id,
                            &config.voted_choice_ids,
                            &choice_ids,
                        )
                        .await?;

                    config.voted_choice_ids = choice_ids;

                    if config.state.live {
                        ctx.exchange_publish(
                            control::exchange::current_room_all_participants(self.room),
                            exchange::Message::Update(poll_id),
                        );
                    }
                } else {
                    ctx.ws_send(Error::InvalidChoiceId);
                }

                Ok(())
            }
            PollsCommand::Finish(finish) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::InsufficientPermissions);

                    return Ok(());
                }

                if self
                    .config
                    .as_ref()
                    .filter(|config| config.state.id == finish.id && !config.state.is_expired())
                    .is_some()
                {
                    // Delete config to stop vote
                    ctx.volatile.storage().delete_polls_state(self.room).await?;

                    ctx.exchange_publish(
                        control::exchange::current_room_all_participants(self.room),
                        exchange::Message::Finish(finish.id),
                    );
                } else {
                    ctx.ws_send(Error::InvalidPollId);
                }

                Ok(())
            }
        }
    }

    async fn on_exchange_message(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        msg: exchange::Message,
    ) -> Result<(), SignalingModuleError> {
        match msg {
            exchange::Message::Started(polls_state) => {
                let id = polls_state.id;

                ctx.ws_send_overwrite_timestamp(
                    PollsEvent::Started(Started {
                        id,
                        topic: polls_state.topic.clone(),
                        live: polls_state.live,
                        multiple_choice: polls_state.multiple_choice,
                        choices: polls_state.choices.clone(),
                        duration: polls_state.duration,
                    }),
                    polls_state.started,
                );

                ctx.add_event_stream(once(
                    sleep(polls_state.duration).map(move |_| ExpiredEvent(id)),
                ));

                self.config = Some(Config {
                    state: polls_state,
                    voted_choice_ids: BTreeSet::new(),
                });

                Ok(())
            }
            exchange::Message::Update(id) => {
                if let Some(config) = &self.config {
                    let results = ctx
                        .volatile
                        .storage()
                        .poll_results(self.room, &config.state)
                        .await?;

                    ctx.ws_send(PollsEvent::LiveUpdate(Results { id, results }));
                }

                Ok(())
            }
            exchange::Message::Finish(id) => {
                if let Some(config) = self.config.take() {
                    let results = ctx
                        .volatile
                        .storage()
                        .poll_results(self.room, &config.state)
                        .await?;

                    ctx.ws_send(PollsEvent::Done(Results { id, results }));
                }

                Ok(())
            }
        }
    }

    async fn cleanup_room(ctx: &mut DestroyContext<'_>, signaling_room_id: SignalingRoomId) {
        if let Err(e) = ctx
            .volatile
            .storage()
            .delete_polls_state(signaling_room_id)
            .await
        {
            log::error!("failed to remove poll state: {}", Report::from_error(e));
        }

        let list = match ctx.volatile.storage().poll_ids(signaling_room_id).await {
            Ok(list) => list,
            Err(e) => {
                log::error!(
                    "failed to get list of poll results to clean up, {}",
                    Report::from_error(e)
                );
                return;
            }
        };

        for id in list {
            if let Err(e) = ctx
                .volatile
                .storage()
                .delete_poll_results(signaling_room_id, id)
                .await
            {
                log::error!(
                    "failed to remove poll results for id {}, {}",
                    id,
                    Report::from_error(e)
                );
            }
        }

        if let Err(e) = ctx
            .volatile
            .storage()
            .delete_poll_ids(signaling_room_id)
            .await
        {
            log::error!(
                "failed to remove closed poll id list: {}",
                Report::from_error(e)
            );
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    state: PollsState,
    voted_choice_ids: BTreeSet<ChoiceId>,
}
