// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `timer` namespace

use super::{Error, Started, Stopped, UpdatedReadyStatus};

/// Outgoing websocket messages
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", tag = "message")
)]
pub enum TimerEvent {
    /// A timer has been started
    Started(Started),
    /// The current timer has been stopped
    Stopped(Stopped),
    /// A participant updated its ready status
    UpdatedReadyStatus(UpdatedReadyStatus),
    /// An error occurred
    Error(Error),
}

impl From<Started> for TimerEvent {
    fn from(value: Started) -> Self {
        Self::Started(value)
    }
}

impl From<Stopped> for TimerEvent {
    fn from(value: Stopped) -> Self {
        Self::Stopped(value)
    }
}

impl From<UpdatedReadyStatus> for TimerEvent {
    fn from(value: UpdatedReadyStatus) -> Self {
        Self::UpdatedReadyStatus(value)
    }
}

impl From<Error> for TimerEvent {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use std::time::SystemTime;

    use chrono::{DateTime, Duration};
    use opentalk_types_common::time::Timestamp;
    use opentalk_types_signaling::ParticipantId;
    use serde_json::json;

    use super::*;
    use crate::{event::StopKind, Kind, TimerConfig, TimerId};

    #[test]
    fn countdown_started() {
        let started_at: Timestamp = DateTime::from(SystemTime::UNIX_EPOCH).into();
        let ends_at = started_at
            .checked_add_signed(Duration::seconds(5))
            .map(Timestamp::from)
            .unwrap();

        let started = TimerEvent::Started(Started {
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

        let started = TimerEvent::Started(Started {
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
        let stopped = TimerEvent::Stopped(Stopped {
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
        let stopped = TimerEvent::Stopped(Stopped {
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
        let stopped = TimerEvent::Error(Error::InsufficientPermissions);

        assert_eq!(
            serde_json::to_value(stopped).unwrap(),
            json!({
                "message": "error",
                "error": "insufficient_permissions",
            }),
        )
    }
}
