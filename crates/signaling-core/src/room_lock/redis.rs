// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use async_trait::async_trait;
use opentalk_r3dlock::{Mutex, MutexGuard};
use snafu::ResultExt;

use super::{LockError, Locking, RoomLock};
use crate::RedisConnection;

pub struct RoomGuard {
    guard: MutexGuard<RoomLock>,
}

#[async_trait(?Send)]
impl Locking<RoomLock> for RedisConnection {
    type Guard = RoomGuard;

    async fn lock(&mut self, room: RoomLock) -> Result<Self::Guard, LockError> {
        // The redlock parameters are set a bit higher than usual to combat
        // contention when a room gets destroyed while a large number of
        // participants are inside it. (e.g. when a breakout room ends)
        let mutex = Mutex::new(room)
            .with_wait_time(Duration::from_millis(20)..Duration::from_millis(60))
            .with_retries(20);
        let guard = mutex
            .lock(self)
            .await
            .whatever_context("TODO: add error variant or associate error type")?;
        Ok(RoomGuard { guard })
    }

    async fn unlock(&mut self, lock: Self::Guard) -> Result<(), LockError> {
        lock.guard
            .unlock(self)
            .await
            .whatever_context("TODO: add error variant or associate error type")?;
        Ok(())
    }
}
