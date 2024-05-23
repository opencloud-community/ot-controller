// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{btree_map::Entry, BTreeMap},
    sync::{Arc, OnceLock, Weak},
};

use async_trait::async_trait;
use futures::lock::{Mutex, OwnedMutexGuard};
use parking_lot::RwLock;

use super::RoomLock;
use crate::{LockError, Locking, VolatileStaticMemoryStorage};

static ROOM_LOCK_STATE: OnceLock<Arc<RwLock<MemoryLock<RoomLock>>>> = OnceLock::new();

#[derive(Debug)]
pub struct VolatileLockGuard<K>(OwnedMutexGuard<()>, K);

pub(super) trait MemoryLocking<Key: Clone + Ord> {
    fn get_state() -> &'static Arc<RwLock<MemoryLock<Key>>>;
}

impl MemoryLocking<RoomLock> for VolatileStaticMemoryStorage {
    fn get_state() -> &'static Arc<RwLock<MemoryLock<RoomLock>>> {
        ROOM_LOCK_STATE.get_or_init(Default::default)
    }
}

#[async_trait(?Send)]
impl<T: MemoryLocking<Key>, Key: Clone + Ord + 'static> Locking<Key> for T {
    type Guard = VolatileLockGuard<Key>;

    async fn lock(&mut self, key: Key) -> Result<Self::Guard, LockError> {
        let lock = Self::get_state().write().get_lock(key.clone());
        let guard = lock.lock_owned().await;

        Ok(VolatileLockGuard(guard, key))
    }

    async fn unlock(&mut self, guard: Self::Guard) -> Result<(), LockError> {
        let VolatileLockGuard(guard, key) = guard;
        let mut memory_locks = Self::get_state().write();
        drop(guard);
        memory_locks.remove_if_unused(key);
        Ok(())
    }
}

#[derive(Debug)]
pub(super) struct MemoryLock<LockKey> {
    locks: BTreeMap<LockKey, Weak<Mutex<()>>>,
}

impl<T> Default for MemoryLock<T> {
    fn default() -> Self {
        Self {
            locks: Default::default(),
        }
    }
}

impl<K: Clone + Ord> MemoryLock<K> {
    fn get_lock(&mut self, key: K) -> Arc<Mutex<()>> {
        match self.locks.entry(key) {
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

    fn remove_if_unused(&mut self, key: K) {
        if let Some(lock) = self.locks.get(&key) {
            if lock.strong_count() == 0 {
                self.locks.remove(&key);
            }
        }
    }
}
