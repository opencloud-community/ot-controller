// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{btree_map::Entry, BTreeMap},
    sync::{Arc, OnceLock, Weak},
};

use async_trait::async_trait;
use either::Either;
use futures::lock::Mutex;
use parking_lot::RwLock;
use snafu::whatever;

use super::{RoomGuard, RoomLocking};
use crate::{LockError, SignalingRoomId, VolatileStaticMemoryStorage};

static ROOM_LOCK_STATE: OnceLock<Arc<RwLock<RoomLocks>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<RoomLocks>> {
    ROOM_LOCK_STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl RoomLocking for VolatileStaticMemoryStorage {
    async fn lock_room(&mut self, room: SignalingRoomId) -> Result<RoomGuard, LockError> {
        let lock = state().write().get_room_lock(room);
        let guard = lock.lock_owned().await;

        Ok(RoomGuard {
            room,
            guard: Either::Left(guard),
        })
    }

    async fn unlock_room(&mut self, RoomGuard { room, guard }: RoomGuard) -> Result<(), LockError> {
        match guard {
            Either::Right(_) => {
                whatever!("Attempted to unlock a redis storage room guard in an in-memory backend")
            }
            Either::Left(guard) => {
                drop(guard);
                state().write().remove_if_unused(room);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub(super) struct RoomLocks {
    locks: BTreeMap<SignalingRoomId, Weak<Mutex<()>>>,
}

impl RoomLocks {
    fn get_room_lock(&mut self, room: SignalingRoomId) -> Arc<Mutex<()>> {
        match self.locks.entry(room) {
            Entry::Vacant(entry) => {
                let mutex = Arc::<Mutex<()>>::default();
                entry.insert(Arc::downgrade(&mutex));
                mutex
            }
            Entry::Occupied(mut entry) => {
                if let Some(mutex) = entry.get().upgrade() {
                    mutex
                } else {
                    let mutex = Arc::<Mutex<()>>::default();
                    entry.insert(Arc::downgrade(&mutex));
                    mutex
                }
            }
        }
    }

    fn remove_if_unused(&mut self, room: SignalingRoomId) {
        if let Some(lock) = self.locks.get(&room) {
            if lock.strong_count() == 0 {
                self.locks.remove(&room);
            }
        }
    }
}
