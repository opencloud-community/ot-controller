// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types::{
    core::{ParticipantId, Timestamp},
    signaling::timer::{Kind, TimerId},
};
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

mod redis;
mod timer_storage;
mod volatile;

pub(crate) use timer_storage::TimerStorage;

/// A timer
///
/// Stores information about a running timer
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToRedisArgs, FromRedisValue)]
#[to_redis_args(serde)]
#[from_redis_value(serde)]
pub(crate) struct Timer {
    /// The timers id
    ///
    /// Used to match expire events to their respective timer
    pub(crate) id: TimerId,
    /// The creator of the timer
    pub(crate) created_by: ParticipantId,
    /// The start of the timer
    ///
    /// Allows us to calculate the passed duration for joining participants
    pub(crate) started_at: Timestamp,
    /// The Timer kind
    pub(crate) kind: Kind,
    /// An optional string tag to flag this timer with a custom style
    pub(crate) style: Option<String>,
    /// The optional title
    pub(crate) title: Option<String>,
    /// Flag to allow/disallow participants to mark themselves as ready
    pub(crate) ready_check_enabled: bool,
}

#[cfg(test)]
mod test_common {
    use opentalk_signaling_core::SignalingRoomId;
    use opentalk_types::{
        core::{ParticipantId, Timestamp},
        signaling::timer::{ready_status::ReadyStatus, Kind, TimerId},
    };
    use pretty_assertions::assert_eq;

    use super::{Timer, TimerStorage};

    const ROOM: SignalingRoomId = SignalingRoomId::nil();

    const ALICE: ParticipantId = ParticipantId::nil();

    pub(super) async fn ready_status(storage: &mut dyn TimerStorage) {
        assert!(storage
            .ready_status_get(ROOM, ALICE)
            .await
            .unwrap()
            .is_none());

        storage.ready_status_set(ROOM, ALICE, true).await.unwrap();

        assert_eq!(
            Some(ReadyStatus { ready_status: true }),
            storage.ready_status_get(ROOM, ALICE).await.unwrap()
        );

        storage.ready_status_delete(ROOM, ALICE).await.unwrap();

        assert!(storage
            .ready_status_get(ROOM, ALICE)
            .await
            .unwrap()
            .is_none());
    }

    pub(super) async fn timer(storage: &mut dyn TimerStorage) {
        let timer = Timer {
            id: TimerId::generate(),
            created_by: ALICE,
            started_at: Timestamp::now(),
            kind: Kind::Stopwatch,
            style: None,
            title: None,
            ready_check_enabled: false,
        };

        assert!(storage.timer_set_if_not_exists(ROOM, &timer).await.unwrap());
        assert_eq!(Some(timer.clone()), storage.timer_get(ROOM).await.unwrap());

        let new_timer = Timer {
            id: TimerId::generate(),
            ..timer.clone()
        };
        assert!(!storage
            .timer_set_if_not_exists(ROOM, &new_timer)
            .await
            .unwrap());
        assert_eq!(Some(timer), storage.timer_get(ROOM).await.unwrap());
    }
}
