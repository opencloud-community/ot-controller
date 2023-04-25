// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use chrono::Utc;
use controller::prelude::*;
use futures::stream::once;
use futures::FutureExt;
use incoming::PollsCommand;
use outgoing::PollsEvent;
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use types::{
    core::Timestamp,
    signaling::{
        polls::{
            event::{Error, Started},
            Choice, ChoiceId, PollId, Results,
        },
        Role,
    },
};

pub mod exchange;
pub mod incoming;
pub mod outgoing;
mod storage;

pub struct ExpiredEvent(PollId);

pub struct Polls {
    room: SignalingRoomId,
    config: Option<Config>,
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for Polls {
    const NAMESPACE: &'static str = "polls";

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
    ) -> Result<Option<Self>> {
        Ok(Some(Self {
            room: ctx.room_id(),
            config: None,
        }))
    }

    async fn on_event(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        event: Event<'_, Self>,
    ) -> Result<()> {
        match event {
            Event::Joined {
                control_data: _,
                frontend_data,
                participants: _,
            } => {
                if let Some(polls_state) = storage::get_state(ctx.redis_conn(), self.room).await? {
                    if let Some(duration) = polls_state.remaining() {
                        let id = polls_state.id;

                        self.config = Some(Config {
                            state: polls_state.clone(),
                            voted: false,
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
            Event::WsMessage(msg) => self.on_ws_message(ctx, msg).await,
            Event::Exchange(msg) => self.on_exchange_message(ctx, msg).await,
            Event::Ext(ExpiredEvent(id)) => {
                if let Some(config) = self.config.as_ref().filter(|config| config.state.id == id) {
                    let results =
                        storage::poll_results(ctx.redis_conn(), self.room, &config.state).await?;

                    ctx.ws_send(PollsEvent::Done(Results { id, results }));
                }

                Ok(())
            }
        }
    }

    async fn on_destroy(self, mut ctx: DestroyContext<'_>) {
        if ctx.destroy_room() {
            if let Err(e) = storage::del_state(ctx.redis_conn(), self.room).await {
                log::error!("failed to remove config from redis: {:?}", e);
            }

            let list = match storage::list_members(ctx.redis_conn(), self.room).await {
                Ok(list) => list,
                Err(e) => {
                    log::error!("failed to get list of poll results to clean up, {:?}", e);
                    return;
                }
            };

            for id in list {
                if let Err(e) = storage::del_results(ctx.redis_conn(), self.room, id).await {
                    log::error!("failed to remove poll results for id {}, {:?}", id, e);
                }
            }
        }
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
    ) -> Result<()> {
        match msg {
            PollsCommand::Start(incoming::Start {
                topic,
                live,
                choices,
                duration,
            }) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(PollsEvent::Error(Error::InsufficientPermissions));

                    return Ok(());
                }

                if self.is_running() {
                    ctx.ws_send(PollsEvent::Error(Error::StillRunning));

                    return Ok(());
                }

                // TODO(k.balt): Minimal duration 2 secs for tests but thats unreasonably low real world applications
                let min = Duration::from_secs(2);
                let max = Duration::from_secs(3600);

                if duration > max || duration < min {
                    ctx.ws_send(PollsEvent::Error(Error::InvalidDuration));

                    return Ok(());
                }

                if !matches!(topic.len(), 2..=100) {
                    ctx.ws_send(PollsEvent::Error(Error::InvalidTopicLength));

                    return Ok(());
                }

                if !matches!(choices.len(), 2..=64) {
                    ctx.ws_send(PollsEvent::Error(Error::InvalidChoiceCount));

                    return Ok(());
                }

                if choices
                    .iter()
                    .any(|content| !matches!(content.len(), 1..=100))
                {
                    ctx.ws_send(PollsEvent::Error(Error::InvalidChoiceDescription));

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
                    choices,
                    started: ctx.timestamp(),
                    duration,
                };

                let set = storage::set_state(ctx.redis_conn(), self.room, &polls_state).await?;

                if !set {
                    ctx.ws_send(PollsEvent::Error(Error::StillRunning));

                    return Ok(());
                }

                storage::list_add(ctx.redis_conn(), self.room, polls_state.id).await?;

                ctx.exchange_publish(
                    control::exchange::current_room_all_participants(self.room),
                    exchange::Message::Started(polls_state),
                );

                Ok(())
            }
            PollsCommand::Vote(incoming::Vote { poll_id, choice_id }) => {
                if let Some(config) = self
                    .config
                    .as_mut()
                    .filter(|config| config.state.id == poll_id && !config.state.is_expired())
                {
                    if config.voted {
                        ctx.ws_send(PollsEvent::Error(Error::VotedAlready));

                        return Ok(());
                    }

                    if config
                        .state
                        .choices
                        .iter()
                        .any(|choice| choice.id == choice_id)
                    {
                        storage::vote(ctx.redis_conn(), self.room, config.state.id, choice_id)
                            .await?;

                        config.voted = true;

                        if config.state.live {
                            ctx.exchange_publish(
                                control::exchange::current_room_all_participants(self.room),
                                exchange::Message::Update(poll_id),
                            );
                        }
                    } else {
                        ctx.ws_send(PollsEvent::Error(Error::InvalidChoiceId));
                    }
                } else {
                    ctx.ws_send(PollsEvent::Error(Error::InvalidPollId));
                }

                Ok(())
            }
            PollsCommand::Finish(finish) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(PollsEvent::Error(Error::InsufficientPermissions));

                    return Ok(());
                }

                if self
                    .config
                    .as_ref()
                    .filter(|config| config.state.id == finish.id && !config.state.is_expired())
                    .is_some()
                {
                    // Delete config from redis to stop vote
                    storage::del_state(ctx.redis_conn(), self.room).await?;

                    ctx.exchange_publish(
                        control::exchange::current_room_all_participants(self.room),
                        exchange::Message::Finish(finish.id),
                    );
                } else {
                    ctx.ws_send(PollsEvent::Error(Error::InvalidPollId));
                }

                Ok(())
            }
        }
    }

    async fn on_exchange_message(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        msg: exchange::Message,
    ) -> Result<()> {
        match msg {
            exchange::Message::Started(polls_state) => {
                let id = polls_state.id;

                ctx.ws_send_overwrite_timestamp(
                    PollsEvent::Started(Started {
                        id,
                        topic: polls_state.topic.clone(),
                        live: polls_state.live,
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
                    voted: false,
                });

                Ok(())
            }
            exchange::Message::Update(id) => {
                if let Some(config) = &self.config {
                    let results =
                        storage::poll_results(ctx.redis_conn(), self.room, &config.state).await?;

                    ctx.ws_send(PollsEvent::LiveUpdate(Results { id, results }));
                }

                Ok(())
            }
            exchange::Message::Finish(id) => {
                if let Some(config) = self.config.take() {
                    let results =
                        storage::poll_results(ctx.redis_conn(), self.room, &config.state).await?;

                    ctx.ws_send(PollsEvent::Done(Results { id, results }));
                }

                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    state: PollsState,
    voted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToRedisArgs, FromRedisValue)]
#[to_redis_args(serde)]
#[from_redis_value(serde)]
pub struct PollsState {
    id: PollId,
    topic: String,
    live: bool,
    choices: Vec<Choice>,
    started: Timestamp,
    #[serde(with = "duration_secs")]
    duration: Duration,
}

impl PollsState {
    fn remaining(&self) -> Option<Duration> {
        let duration = chrono::Duration::from_std(self.duration)
            .expect("duration as secs should never be larger than i64::MAX");

        let expire = (*self.started) + duration;
        let now = Utc::now();

        // difference will be negative duration if expired.
        // Conversion to std duration will fail -> returning None
        (expire - now).to_std().ok()
    }

    fn is_expired(&self) -> bool {
        self.remaining().is_none()
    }
}

mod duration_secs {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        u64::deserialize(deserializer).map(Duration::from_secs)
    }

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }
}

pub fn register(controller: &mut controller::Controller) {
    controller.signaling.add_module::<Polls>(());
}
