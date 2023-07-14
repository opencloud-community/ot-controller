// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use chrono::{self, Utc};
use futures::{stream::once, FutureExt};
use outgoing::StopKind;
use redis::{self, FromRedisValue, RedisResult};
use redis_args::ToRedisArgs;
use serde::Deserialize;
use serde::Serialize;
use signaling_core::SignalingModuleInitData;
use signaling_core::{
    control, DestroyContext, Event, InitContext, ModuleContext, SignalingModule, SignalingRoomId,
};
use storage::ready_status::ReadyStatus;
use tokio::time::sleep;
use types::{
    core::{ParticipantId, Timestamp},
    signaling::Role,
};
use uuid::Uuid;

use std::fmt;
use std::str::from_utf8;
use std::str::FromStr;

pub mod exchange;
pub mod incoming;
pub mod outgoing;
mod storage;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, ToRedisArgs)]
#[to_redis_args(fmt)]
pub struct TimerId(pub Uuid);

impl TimerId {
    /// Create a ZERO TimerId, e.g. for testing purposes
    pub const fn nil() -> Self {
        Self(Uuid::nil())
    }

    /// Create a TimerId from a number, e.g. for testing purposes
    pub const fn from_u128(id: u128) -> Self {
        Self(Uuid::from_u128(id))
    }

    /// Generate a new random TimerId
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}

impl FromRedisValue for TimerId {
    fn from_redis_value(v: &redis::Value) -> RedisResult<Self> {
        match v {
            redis::Value::Data(bytes) => {
                Uuid::from_str(from_utf8(bytes)?).map(Self).map_err(|_| {
                    redis::RedisError::from((
                        redis::ErrorKind::TypeError,
                        "invalid data for TimerId",
                    ))
                })
            }
            _ => RedisResult::Err(redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "invalid data type for TimerId",
            ))),
        }
    }
}

impl fmt::Display for TimerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// The expiry event for a timer
pub struct ExpiredEvent {
    timer_id: TimerId,
}

pub struct Timer {
    pub room_id: SignalingRoomId,
    pub participant_id: ParticipantId,
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for Timer {
    const NAMESPACE: &'static str = "timer";

    type Params = ();

    type Incoming = incoming::Message;

    type Outgoing = outgoing::Message;

    type ExchangeMessage = exchange::Event;

    type ExtEvent = ExpiredEvent;

    type FrontendData = TimerStatus;

    type PeerFrontendData = ReadyStatus;

    async fn init(
        ctx: InitContext<'_, Self>,
        _params: &Self::Params,
        _protocol: &'static str,
    ) -> Result<Option<Self>> {
        Ok(Some(Self {
            room_id: ctx.room_id(),
            participant_id: ctx.participant_id(),
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
                participants,
            } => {
                let timer = storage::timer::get(ctx.redis_conn(), self.room_id).await?;

                let timer = match timer {
                    Some(timer) => timer,
                    None => return Ok(()),
                };

                let ready_status = if timer.ready_check_enabled {
                    Some(
                        storage::ready_status::get(
                            ctx.redis_conn(),
                            self.room_id,
                            self.participant_id,
                        )
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
                    let ready_status =
                        storage::ready_status::get(ctx.redis_conn(), self.room_id, *participant_id)
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
                if let Some(timer) = storage::timer::get(ctx.redis_conn(), self.room_id).await? {
                    if timer.id == expired.timer_id {
                        self.stop_current_timer(&mut ctx, StopKind::Expired, None)
                            .await?;
                    }
                }
            }
            Event::ParticipantJoined(id, data) => {
                // As in Event::Joined, don't attach any timer-related information if no timer is active.
                let timer = storage::timer::get(ctx.redis_conn(), self.room_id).await?;

                let timer = match timer {
                    Some(timer) => timer,
                    None => return Ok(()),
                };

                if !timer.ready_check_enabled {
                    return Ok(());
                }
                let ready_status = storage::ready_status::get(ctx.redis_conn(), self.room_id, id)
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

    async fn build_params(_init: &SignalingModuleInitData) -> Result<Option<Self::Params>> {
        Ok(Some(()))
    }
}

impl Timer {
    /// Handle incoming websocket messages
    async fn handle_ws_message(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        msg: incoming::Message,
    ) -> Result<()> {
        match msg {
            incoming::Message::Start(start) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(outgoing::Message::Error(
                        outgoing::Error::InsufficientPermissions,
                    ));
                    return Ok(());
                }

                let timer_id = TimerId(Uuid::new_v4());

                let started_at = ctx.timestamp();

                // determine the end time at the start of the timer to later calculate the remaining duration for joining participants
                let kind = match start.kind {
                    incoming::Kind::Countdown { duration } => {
                        let duration = match duration.try_into() {
                            Ok(duration) => duration,
                            Err(_) => {
                                ctx.ws_send(outgoing::Message::Error(
                                    outgoing::Error::InvalidDuration,
                                ));

                                return Ok(());
                            }
                        };

                        match started_at.checked_add_signed(chrono::Duration::seconds(duration)) {
                            Some(ends_at) => Kind::Countdown {
                                ends_at: Timestamp::from(ends_at),
                            },
                            None => {
                                log::error!("DateTime overflow in timer module");
                                ctx.ws_send(outgoing::Message::Error(
                                    outgoing::Error::InvalidDuration,
                                ));

                                return Ok(());
                            }
                        }
                    }
                    incoming::Kind::Stopwatch => Kind::Stopwatch,
                };

                let timer = storage::timer::Timer {
                    id: timer_id,
                    created_by: self.participant_id,
                    started_at,
                    kind,
                    style: start.style.clone(),
                    title: start.title.clone(),
                    ready_check_enabled: start.enable_ready_check,
                };

                if !storage::timer::set_if_not_exists(ctx.redis_conn(), self.room_id, &timer)
                    .await?
                {
                    ctx.ws_send(outgoing::Message::Error(
                        outgoing::Error::TimerAlreadyRunning,
                    ));
                    return Ok(());
                }

                let started = outgoing::Started {
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
            incoming::Message::Stop(stop) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(outgoing::Message::Error(
                        outgoing::Error::InsufficientPermissions,
                    ));
                    return Ok(());
                }

                match storage::timer::get(ctx.redis_conn(), self.room_id).await? {
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
            incoming::Message::UpdateReadyStatus(update_ready_status) => {
                if let Some(timer) = storage::timer::get(ctx.redis_conn(), self.room_id).await? {
                    if timer.ready_check_enabled && timer.id == update_ready_status.timer_id {
                        storage::ready_status::set(
                            ctx.redis_conn(),
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
    ) -> Result<()> {
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

                ctx.ws_send(outgoing::Message::Started(started));
            }
            exchange::Event::Stop(stopped) => {
                // remove the participants ready status when receiving 'stopped'
                storage::ready_status::delete(ctx.redis_conn(), self.room_id, self.participant_id)
                    .await?;

                ctx.ws_send(outgoing::Message::Stopped(stopped));
            }
            exchange::Event::UpdateReadyStatus(update_ready_status) => {
                if let Some(ready_status) = storage::ready_status::get(
                    ctx.redis_conn(),
                    self.room_id,
                    update_ready_status.participant_id,
                )
                .await?
                {
                    ctx.ws_send(outgoing::Message::UpdatedReadyStatus(
                        outgoing::UpdatedReadyStatus {
                            timer_id: update_ready_status.timer_id,
                            participant_id: update_ready_status.participant_id,
                            status: ready_status.ready_status,
                        },
                    ))
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
    ) -> Result<()> {
        let timer = match storage::timer::delete(ctx.redis_conn(), self.room_id).await? {
            Some(timer) => timer,
            // there was no key to delete because the timer was not running
            None => return Ok(()),
        };

        ctx.exchange_publish(
            control::exchange::current_room_all_participants(self.room_id),
            exchange::Event::Stop(outgoing::Stopped {
                timer_id: timer.id,
                kind: reason,
                reason: message,
            }),
        );

        Ok(())
    }
}

/// The different timer variations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Kind {
    /// The timer continues to run until a moderator stops it.
    Stopwatch,
    /// The timer continues to run until its duration expires or if a moderator stops it beforehand.
    Countdown { ends_at: Timestamp },
}

/// Status of a currently active timer
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimerConfig {
    /// The timer id
    pub timer_id: TimerId,
    /// start time of the timer
    pub started_at: Timestamp,
    /// Timer kind
    #[serde(flatten)]
    pub kind: Kind,
    /// Style to use for the timer. Set by the sender.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// The optional title of the timer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Flag to allow/disallow participants to mark themselves as ready
    pub ready_check_enabled: bool,
}

/// Status of and belonging to a currently active timer
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimerStatus {
    #[serde(flatten)]
    pub config: TimerConfig,
    /// Flag to indicate that the current participant has marked themselves as ready
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ready_status: Option<bool>,
}

#[cfg(test)]
mod test {
    use std::time::SystemTime;

    use super::*;
    use crate::Kind;
    use chrono::{DateTime, Duration};
    use test_util::assert_eq_json;
    use types::core::Timestamp;

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
