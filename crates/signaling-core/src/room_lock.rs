// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use either::Either;
use futures::lock::OwnedMutexGuard;
use opentalk_r3dlock::Error;
use redis_args::ToRedisArgs;
use snafu::Snafu;

use crate::{SignalingRoomId, VolatileStorage};

mod redis;
mod volatile;

/// Key used for the lock over the room participants set
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:participants.lock")]
pub struct RoomLock {
    pub room: SignalingRoomId,
}

pub struct RoomGuard {
    room: SignalingRoomId,
    guard: Either<OwnedMutexGuard<()>, opentalk_r3dlock::MutexGuard<RoomLock>>,
}

#[async_trait(?Send)]
pub trait RoomLocking {
    /// Lock the room for exclusive access.
    ///
    /// Must be locked when joining and leaving the room.
    /// This allows for cleanups when the last user leaves without anyone joining.
    async fn lock_room(&mut self, room: SignalingRoomId) -> Result<RoomGuard, LockError>;
    async fn unlock_room(&mut self, guard: RoomGuard) -> Result<(), LockError>;
}

#[derive(Debug, Snafu)]
pub enum LockError {
    /// Failed to acquire the lock within the given time/resource constraints.
    Locked,

    /// There was an internal error while trying to acquire the lock
    Internal { message: String },

    #[snafu(whatever)]
    Other {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, Some)))]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl From<Error> for LockError {
    fn from(value: Error) -> Self {
        match value {
            Error::FailedToUnlock | Error::AlreadyExpired => Self::Internal {
                message: value.to_string(),
            },
            Error::CouldNotAcquireLock => Self::Locked,
            Error::Redis { ref source } => Self::Internal {
                message: format!("{}: {}", value, source),
            },
        }
    }
}

pub trait RoomLockingProvider {
    fn room_locking(&mut self) -> &mut dyn RoomLocking;
}

impl RoomLockingProvider for VolatileStorage {
    fn room_locking(&mut self) -> &mut dyn RoomLocking {
        match self.as_mut() {
            Either::Left(v) => v,
            Either::Right(v) => v,
        }
    }
}
