// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{self, Utc};
use either::Either;
use futures::{stream::once, FutureExt};
use opentalk_signaling_core::{
    control, DestroyContext, Event, InitContext, ModuleContext, SignalingModule,
    SignalingModuleError, SignalingModuleInitData, SignalingRoomId, VolatileStorage,
};
use opentalk_types::signaling::timer::{
    event::{self, Error, StopKind, UpdatedReadyStatus},
    ready_status::ReadyStatus,
    status::TimerStatus,
};
use opentalk_types_common::time::Timestamp;
use opentalk_types_signaling::{ParticipantId, Role};
use opentalk_types_signaling_timer::{
    command::{self, TimerCommand},
    event::Started,
    Kind, TimerConfig, TimerId, NAMESPACE,
};
use storage::TimerStorage;
use tokio::time::sleep;
use uuid::Uuid;

pub mod exchange;
mod storage;

/// The expiry event for a timer
pub struct ExpiredEvent {
    timer_id: TimerId,
}

pub struct Timer {
    pub room_id: SignalingRoomId,
    pub participant_id: ParticipantId,
}

trait TimerStorageProvider {
    fn storage(&mut self) -> &mut dyn TimerStorage;
}

impl TimerStorageProvider for VolatileStorage {
    fn storage(&mut self) -> &mut dyn TimerStorage {
        match self.as_mut() {
            Either::Left(v) => v,
            Either::Right(v) => v,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for Timer {
    const NAMESPACE: &'static str = NAMESPACE;

    type Params = ();

    type Incoming = TimerCommand;

    type Outgoing = event::Message;

    type ExchangeMessage = exchange::Event;

    type ExtEvent = ExpiredEvent;

    type FrontendData = TimerStatus;

    type PeerFrontendData = ReadyStatus;

    async fn init(
        ctx: InitContext<'_, Self>,
        _params: &Self::Params,
        _protocol: &'static str,
    ) -> Result<Option<Self>, SignalingModuleError> {
        Ok(Some(Self {
            room_id: ctx.room_id(),
            participant_id: ctx.participant_id(),
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
                participants,
            } => {
                let timer = ctx.volatile.storage().timer_get(self.room_id).await?;

                let timer = match timer {
                    Some(timer) => timer,
                    None => return Ok(()),
                };

                let ready_status = if timer.ready_check_enabled {
                    Some(
                        ctx.volatile
                            .storage()
                            .ready_status_get(self.room_id, self.participant_id)
                            .await?
                            .unwrap_or_default()
                            .ready_status,
                    )
                } else {
                    None
                };

                *frontend_data = Some(TimerStatus {
                    config: TimerConfig {
                        timer_id: timer.id,
                        started_at: timer.started_at,
                        kind: timer.kind,
                        style: timer.style,
                        title: timer.title,
                        ready_check_enabled: timer.ready_check_enabled,
                    },
                    ready_status,
                });

                if let Kind::Countdown { ends_at } = timer.kind {
                    ctx.add_event_stream(once(
                        sleep(
                            ends_at
                                .signed_duration_since(Utc::now())
                                .to_std()
                                .unwrap_or_default(),
                        )
                        .map(move |_| ExpiredEvent { timer_id: timer.id }),
                    ));
                }

                if !timer.ready_check_enabled {
                    return Ok(());
                }
                for (participant_id, status) in participants {
                    let ready_status = ctx
                        .volatile
                        .storage()
                        .ready_status_get(self.room_id, *participant_id)
                        .await?
                        .unwrap_or_default();

                    *status = Some(ready_status);
                }
            }
            Event::WsMessage(msg) => self.handle_ws_message(&mut ctx, msg).await?,
            Event::Exchange(event) => {
                self.handle_rmq_message(&mut ctx, event).await?;
            }
            Event::Ext(expired) => {
                if let Some(timer) = ctx.volatile.storage().timer_get(self.room_id).await? {
                    if timer.id == expired.timer_id {
                        self.stop_current_timer(&mut ctx, StopKind::Expired, None)
                            .await?;
                    }
                }
            }
            Event::ParticipantJoined(id, data) => {
                // As in Event::Joined, don't attach any timer-related information if no timer is active.
                let timer = ctx.volatile.storage().timer_get(self.room_id).await?;

                let timer = match timer {
                    Some(timer) => timer,
                    None => return Ok(()),
                };

                if !timer.ready_check_enabled {
                    return Ok(());
                }
                let ready_status = ctx
                    .volatile
                    .storage()
                    .ready_status_get(self.room_id, id)
                    .await?
                    .unwrap_or_default();

                *data = Some(ready_status);
            }
            // Unused
            Event::Leaving
            | Event::RaiseHand
            | Event::LowerHand
            | Event::ParticipantUpdated(_, _)
            | Event::ParticipantLeft(_)
            | Event::RoleUpdated(_) => {}
        }

        Ok(())
    }

    async fn on_destroy(self, _ctx: DestroyContext<'_>) {}

    async fn build_params(
        _init: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
        Ok(Some(()))
    }
}

impl Timer {
    /// Handle incoming websocket messages
    async fn handle_ws_message(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        msg: TimerCommand,
    ) -> Result<(), SignalingModuleError> {
        match msg {
            TimerCommand::Start(start) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::InsufficientPermissions);
                    return Ok(());
                }

                let timer_id = TimerId(Uuid::new_v4());

                let started_at = ctx.timestamp();

                // determine the end time at the start of the timer to later calculate the remaining duration for joining participants
                let kind = match start.kind {
                    command::Kind::Countdown { duration } => {
                        let duration = match duration.try_into() {
                            Ok(duration) => duration,
                            Err(_) => {
                                ctx.ws_send(Error::InvalidDuration);

                                return Ok(());
                            }
                        };

                        match started_at.checked_add_signed(chrono::Duration::seconds(duration)) {
                            Some(ends_at) => Kind::Countdown {
                                ends_at: Timestamp::from(ends_at),
                            },
                            None => {
                                log::error!("DateTime overflow in timer module");
                                ctx.ws_send(Error::InvalidDuration);

                                return Ok(());
                            }
                        }
                    }
                    command::Kind::Stopwatch => Kind::Stopwatch,
                };

                let timer = storage::Timer {
                    id: timer_id,
                    created_by: self.participant_id,
                    started_at,
                    kind,
                    style: start.style.clone(),
                    title: start.title.clone(),
                    ready_check_enabled: start.enable_ready_check,
                };

                if !ctx
                    .volatile
                    .storage()
                    .timer_set_if_not_exists(self.room_id, &timer)
                    .await?
                {
                    ctx.ws_send(Error::TimerAlreadyRunning);
                    return Ok(());
                }

                let started = Started {
                    config: TimerConfig {
                        timer_id,
                        started_at,
                        kind,
                        style: start.style,
                        title: start.title,
                        ready_check_enabled: start.enable_ready_check,
                    },
                };

                ctx.exchange_publish(
                    control::exchange::current_room_all_participants(self.room_id),
                    exchange::Event::Start(started),
                );
            }
            TimerCommand::Stop(stop) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::InsufficientPermissions);
                    return Ok(());
                }

                match ctx.volatile.storage().timer_get(self.room_id).await? {
                    Some(timer) => {
                        if timer.id != stop.timer_id {
                            // Invalid timer id
                            return Ok(());
                        }
                    }
                    None => {
                        // no timer active
                        return Ok(());
                    }
                }

                self.stop_current_timer(
                    ctx,
                    StopKind::ByModerator(self.participant_id),
                    stop.reason,
                )
                .await?;
            }
            TimerCommand::UpdateReadyStatus(update_ready_status) => {
                if let Some(timer) = ctx.volatile.storage().timer_get(self.room_id).await? {
                    if timer.ready_check_enabled && timer.id == update_ready_status.timer_id {
                        ctx.volatile
                            .storage()
                            .ready_status_set(
                                self.room_id,
                                self.participant_id,
                                update_ready_status.status,
                            )
                            .await?;

                        ctx.exchange_publish(
                            control::exchange::current_room_all_participants(self.room_id),
                            exchange::Event::UpdateReadyStatus(exchange::UpdateReadyStatus {
                                timer_id: timer.id,
                                participant_id: self.participant_id,
                            }),
                        );
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_rmq_message(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        event: exchange::Event,
    ) -> Result<(), SignalingModuleError> {
        match event {
            exchange::Event::Start(started) => {
                if let Kind::Countdown { ends_at } = started.config.kind {
                    ctx.add_event_stream(once(
                        sleep(
                            ends_at
                                .signed_duration_since(Utc::now())
                                .to_std()
                                .unwrap_or_default(),
                        )
                        .map(move |_| ExpiredEvent {
                            timer_id: started.config.timer_id,
                        }),
                    ));
                }

                ctx.ws_send(started);
            }
            exchange::Event::Stop(stopped) => {
                // remove the participants ready status when receiving 'stopped'
                ctx.volatile
                    .storage()
                    .ready_status_delete(self.room_id, self.participant_id)
                    .await?;

                ctx.ws_send(stopped);
            }
            exchange::Event::UpdateReadyStatus(update_ready_status) => {
                if let Some(ready_status) = ctx
                    .volatile
                    .storage()
                    .ready_status_get(self.room_id, update_ready_status.participant_id)
                    .await?
                {
                    ctx.ws_send(UpdatedReadyStatus {
                        timer_id: update_ready_status.timer_id,
                        participant_id: update_ready_status.participant_id,
                        status: ready_status.ready_status,
                    })
                }
            }
        }

        Ok(())
    }

    /// Stop the current timer and publish a [`outgoing::Stopped`] message to all participants
    ///
    /// Does not send the [`outgoing::Stopped`] message when there is no timer running
    async fn stop_current_timer(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        reason: StopKind,
        message: Option<String>,
    ) -> Result<(), SignalingModuleError> {
        let timer = match ctx.volatile.storage().timer_delete(self.room_id).await? {
            Some(timer) => timer,
            // there was no key to delete because the timer was not running
            None => return Ok(()),
        };

        ctx.exchange_publish(
            control::exchange::current_room_all_participants(self.room_id),
            exchange::Event::Stop(event::Stopped {
                timer_id: timer.id,
                kind: reason,
                reason: message,
            }),
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use chrono::{DateTime, Duration};
    use opentalk_test_util::assert_eq_json;
    use opentalk_types::signaling::timer::status::TimerStatus;

    use super::*;
    use crate::Kind;

    #[test]
    fn timer_status_without_ready_status() {
        let started_at: Timestamp = DateTime::from(SystemTime::UNIX_EPOCH).into();
        let ends_at = started_at
            .checked_add_signed(Duration::seconds(5))
            .map(Timestamp::from)
            .unwrap();

        let timer_status = TimerStatus {
            config: TimerConfig {
                timer_id: TimerId::nil(),
                started_at,
                kind: Kind::Countdown { ends_at },
                style: Some("coffee_break".into()),
                title: None,
                ready_check_enabled: false,
            },
            ready_status: None,
        };

        assert_eq_json!(timer_status,
        {
            "timer_id": "00000000-0000-0000-0000-000000000000",
            "started_at": "1970-01-01T00:00:00Z",
            "kind": "countdown",
            "style": "coffee_break",
            "ready_check_enabled": false,
            "ends_at": "1970-01-01T00:00:05Z",
        });
    }

    #[test]
    fn timer_status_with_ready_status_true() {
        let started_at: Timestamp = DateTime::from(SystemTime::UNIX_EPOCH).into();
        let ends_at = started_at
            .checked_add_signed(Duration::seconds(5))
            .map(Timestamp::from)
            .unwrap();

        let timer_status = TimerStatus {
            config: TimerConfig {
                timer_id: TimerId::nil(),
                started_at,
                kind: Kind::Countdown { ends_at },
                style: Some("coffee_break".into()),
                title: None,
                ready_check_enabled: true,
            },
            ready_status: Some(true),
        };

        assert_eq_json!(timer_status,
        {
            "timer_id": "00000000-0000-0000-0000-000000000000",
            "started_at": "1970-01-01T00:00:00Z",
            "kind": "countdown",
            "style": "coffee_break",
            "ready_check_enabled": true,
            "ready_status": true,
            "ends_at": "1970-01-01T00:00:05Z",
        });
    }

    #[test]
    fn timer_status_with_ready_status_false() {
        let started_at: Timestamp = DateTime::from(SystemTime::UNIX_EPOCH).into();
        let ends_at = started_at
            .checked_add_signed(Duration::seconds(5))
            .map(Timestamp::from)
            .unwrap();

        let timer_status = TimerStatus {
            config: TimerConfig {
                timer_id: TimerId::nil(),
                started_at,
                kind: Kind::Countdown { ends_at },
                style: Some("coffee_break".into()),
                title: None,
                ready_check_enabled: true,
            },
            ready_status: Some(false),
        };

        assert_eq_json!(timer_status,
        {
            "timer_id": "00000000-0000-0000-0000-000000000000",
            "started_at": "1970-01-01T00:00:00Z",
            "kind": "countdown",
            "style": "coffee_break",
            "ready_check_enabled": true,
            "ready_status": false,
            "ends_at": "1970-01-01T00:00:05Z",
        });
    }
}
