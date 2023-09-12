// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `timer` namespace

use crate::core::ParticipantId;
#[allow(unused_imports)]
use crate::imports::*;

use super::{TimerConfig, TimerId};

/// Outgoing websocket messages
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize),
    serde(rename_all = "snake_case", tag = "message")
)]
pub enum Message {
    /// A timer has been started
    Started(Started),
    /// The current timer has been stopped
    Stopped(Stopped),
    /// A participant updated its ready status
    UpdatedReadyStatus(UpdatedReadyStatus),
    /// An error occurred
    Error(Error),
}

/// A timer has been started
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Started {
    /// Config of the started timer
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub config: TimerConfig,
}

impl From<Started> for Message {
    fn from(value: Started) -> Self {
        Self::Started(value)
    }
}

/// The current timer has been stopped
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Stopped {
    /// The timer id
    pub timer_id: TimerId,
    /// The stop kind
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub kind: StopKind,
    /// An optional reason to all participants. Set by moderator
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub reason: Option<String>,
}

impl From<Stopped> for Message {
    fn from(value: Stopped) -> Self {
        Self::Stopped(value)
    }
}

/// The stop reason
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case", tag = "kind", content = "participant_id")
)]
pub enum StopKind {
    /// The timer has been stopped by a moderator
    ByModerator(ParticipantId),
    /// The timers duration has expired
    Expired,
}

/// Update the ready status
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UpdatedReadyStatus {
    /// The timer id that the update is for
    pub timer_id: TimerId,
    /// The participant that updated its status
    pub participant_id: ParticipantId,
    /// The new status
    pub status: bool,
}

impl From<UpdatedReadyStatus> for Message {
    fn from(value: UpdatedReadyStatus) -> Self {
        Self::UpdatedReadyStatus(value)
    }
}

/// Errors from the `timer` module namespace
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize),
    serde(rename_all = "snake_case", tag = "error")
)]
pub enum Error {
    /// An invalid timer duration has been configured
    InvalidDuration,
    /// The requesting user has insufficient permissions
    InsufficientPermissions,
    /// A timer is already running
    TimerAlreadyRunning,
}

impl From<Error> for Message {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
}

#[cfg(test)]
mod test {
    use std::time::SystemTime;

    use crate::{core::Timestamp, signaling::timer::Kind};

    use super::*;
    use chrono::{DateTime, Duration};
    use serde_json::json;

    #[test]
    fn countdown_started() {
        let started_at: Timestamp = DateTime::from(SystemTime::UNIX_EPOCH).into();
        let ends_at = started_at
            .checked_add_signed(Duration::seconds(5))
            .map(Timestamp::from)
            .unwrap();

        let started = Message::Started(Started {
            config: TimerConfig {
                timer_id: TimerId::nil(),
                started_at,
                kind: Kind::Countdown { ends_at },
                style: Some("coffee_break".into()),
                title: None,
                ready_check_enabled: true,
            },
        });

        assert_eq!(
            serde_json::to_value(started).unwrap(),
            json!({
                "message": "started",
                "timer_id": "00000000-0000-0000-0000-000000000000",
                "started_at": "1970-01-01T00:00:00Z",
                "kind": "countdown",
                "ends_at": "1970-01-01T00:00:05Z",
                "style": "coffee_break",
                "ready_check_enabled": true
            }),
        );
    }

    #[test]
    fn stopwatch_started() {
        let started_at: Timestamp = DateTime::from(SystemTime::UNIX_EPOCH).into();

        let started = Message::Started(Started {
            config: TimerConfig {
                timer_id: TimerId::nil(),
                started_at,
                kind: Kind::Stopwatch,
                style: None,
                title: Some("Testing the timer!".into()),
                ready_check_enabled: false,
            },
        });

        assert_eq!(
            serde_json::to_value(started).unwrap(),
            json!({
                "message": "started",
                "timer_id": "00000000-0000-0000-0000-000000000000",
                "started_at": "1970-01-01T00:00:00Z",
                "kind": "stopwatch",
                "title": "Testing the timer!",
                "ready_check_enabled": false
            }),
        )
    }

    #[test]
    fn stopped_by_moderator() {
        let stopped = Message::Stopped(Stopped {
            timer_id: TimerId::nil(),
            kind: StopKind::ByModerator(ParticipantId::nil()),
            reason: Some("A good reason!".into()),
        });

        assert_eq!(
            serde_json::to_value(stopped).unwrap(),
            json!({
                "message": "stopped",
                "timer_id": "00000000-0000-0000-0000-000000000000",
                "kind": "by_moderator",
                "participant_id": "00000000-0000-0000-0000-000000000000",
                "reason": "A good reason!"
            }),
        )
    }

    #[test]
    fn expired() {
        let stopped = Message::Stopped(Stopped {
            timer_id: TimerId::nil(),
            kind: StopKind::Expired,
            reason: None,
        });

        assert_eq!(
            serde_json::to_value(stopped).unwrap(),
            json!({
                "message": "stopped",
                "timer_id": "00000000-0000-0000-0000-000000000000",
                "kind": "expired",
            }),
        )
    }

    #[test]
    fn error_insufficient_permission() {
        let stopped = Message::Error(Error::InsufficientPermissions);

        assert_eq!(
            serde_json::to_value(stopped).unwrap(),
            json!({
                "message": "error",
                "error": "insufficient_permissions",
            }),
        )
    }
}
