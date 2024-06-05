// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use async_trait::async_trait;
use either::Either;
use opentalk_r3dlock::Mutex;
use snafu::whatever;

use super::{LockError, RoomGuard, RoomLock, RoomLocking};
use crate::{RedisConnection, SignalingRoomId};

#[async_trait(?Send)]
impl RoomLocking for RedisConnection {
    async fn lock_room(&mut self, room: SignalingRoomId) -> Result<RoomGuard, LockError> {
        // The redlock parameters are set a bit higher than usual to combat
        // contention when a room gets destroyed while a large number of
        // participants are inside it. (e.g. when a breakout room ends)
        let mutex = Mutex::new(RoomLock { room })
            .with_wait_time(Duration::from_millis(20)..Duration::from_millis(60))
            .with_retries(20);
        let guard = mutex.lock(self).await?;
        Ok(RoomGuard {
            room,
            guard: Either::Right(guard),
        })
    }

    async fn unlock_room(&mut self, lock: RoomGuard) -> Result<(), LockError> {
        match lock.guard {
            Either::Right(guard) => {
                guard.unlock(self).await?;
            }
            Either::Left(_) => {
                whatever!("Attempted to unlock a in-memory storage room guard in a redis backend")
            }
        }
        Ok(())
    }
}
