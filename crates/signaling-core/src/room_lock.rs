// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use redis_args::ToRedisArgs;
use snafu::Snafu;

use crate::SignalingRoomId;

mod redis;
mod volatile;

/// Key used for the lock over the room participants set
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:participants.lock")]
pub struct RoomLock {
    pub room: SignalingRoomId,
}

impl From<SignalingRoomId> for RoomLock {
    fn from(room: SignalingRoomId) -> Self {
        RoomLock { room }
    }
}

#[async_trait(?Send)]
pub trait Locking<Key> {
    type Guard;

    /// Lock the room for exclusive access.
    ///
    /// Must be locked when joining and leaving the room.
    /// This allows for cleanups when the last user leaves without anyone joining.
    async fn lock(&mut self, key: Key) -> Result<Self::Guard, LockError>;
    async fn unlock(&mut self, guard: Self::Guard) -> Result<(), LockError>;
}

#[derive(Debug, Snafu)]
pub enum LockError {
    /// Failed to acquire the lock within the given time/resource constraints.
    Locked,

    /// There was an internal error while trying to acquire the lock
    Internal,

    #[snafu(whatever)]
    Other {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, Some)))]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}
