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

pub(crate) mod ready_status {
    pub(crate) use super::redis::ready_status_delete as delete;
}

pub(crate) mod timer {
    pub(crate) use super::redis::{
        timer_delete as delete, timer_get as get, timer_set_if_not_exists as set_if_not_exists,
    };
}

/// A timer
///
/// Stores information about a running timer
#[derive(Debug, Serialize, Deserialize, ToRedisArgs, FromRedisValue)]
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
    use opentalk_types::{core::ParticipantId, signaling::timer::ready_status::ReadyStatus};
    use pretty_assertions::assert_eq;

    use super::TimerStorage;

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
    }
}
