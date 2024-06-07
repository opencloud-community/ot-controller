// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::HashMap, hash::Hash, time::Duration};

use crate::ExpiringData;

#[derive(Debug, Clone)]
pub struct ExpiringDataHashMap<K, V>
where
    K: PartialEq + Eq + Hash,
    V: Clone,
{
    data: HashMap<K, ExpiringData<V>>,
}

impl<K, V> Default for ExpiringDataHashMap<K, V>
where
    K: PartialEq + Eq + Hash,
    V: Clone,
{
    fn default() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl<K, V> ExpiringDataHashMap<K, V>
where
    K: PartialEq + Eq + Hash,
    V: Clone,
{
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(key).and_then(ExpiringData::value)
    }

    pub fn insert_with_expiry(&mut self, key: K, value: V, expires_after: Duration) -> bool {
        self.data
            .insert(key, ExpiringData::new_ex(value, expires_after))
            .is_some()
    }

    /// Insert an entry with an expiry if it doesn't exist yet.
    ///
    /// If an entry with that key already exists, it will not be modified in any
    /// way, also the expiry time remains unchanged.
    /// Returns `true` if the entry was created, `false` if the key was already occupied.
    pub fn insert_with_expiry_if_not_exists(
        &mut self,
        key: K,
        value: V,
        expires_after: Duration,
    ) -> bool {
        match self.data.get(&key).map(|v| v.is_expired()) {
            Some(true) | None => {
                self.data
                    .insert(key, ExpiringData::new_ex(value, expires_after));
                true
            }
            Some(false) => false,
        }
    }

    pub fn update_expiry(&mut self, key: &K, expires_after: Duration) -> bool {
        if let Some(data) = self.data.get_mut(key) {
            if !data.is_expired() {
                data.set_expiry(expires_after);
                return true;
            }
        }
        false
    }

    pub fn cleanup_expired(&mut self) {
        self.data.retain(|_k, v| !v.is_expired());
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.data.remove(key).and_then(ExpiringData::take)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use pretty_assertions::assert_eq;

    use crate::ExpiringDataHashMap;

    #[test]
    fn insert_with_expiry() {
        let mut map = ExpiringDataHashMap::new();

        map.insert_with_expiry(1, "hello", Duration::from_millis(4));

        assert_eq!(map.get(&1), Some(&"hello"));

        std::thread::sleep(Duration::from_millis(10));

        assert!(map.get(&1).is_none());
    }

    #[test]
    fn insert_with_expiry_if_not_exists() {
        let mut map = ExpiringDataHashMap::new();

        map.insert_with_expiry(1, "hello", Duration::from_millis(4));

        assert_eq!(map.get(&1), Some(&"hello"));
        assert!(!map.insert_with_expiry_if_not_exists(1, "world", Duration::from_millis(20)));

        std::thread::sleep(Duration::from_millis(10));

        assert!(map.insert_with_expiry_if_not_exists(1, "world", Duration::from_millis(20)));
    }

    #[test]
    fn update_expiry() {
        let mut map = ExpiringDataHashMap::new();

        map.insert_with_expiry(1, "hello", Duration::from_millis(1000));

        std::thread::sleep(Duration::from_millis(10));

        map.update_expiry(&1, Duration::from_millis(3));
        assert!(map.get(&1).is_some());

        std::thread::sleep(Duration::from_millis(10));

        assert!(map.get(&1).is_none());
    }

    #[test]
    fn remove() {
        let mut map = ExpiringDataHashMap::new();

        map.insert_with_expiry(1, "hello", Duration::from_millis(1000));
        map.insert_with_expiry(2, "world", Duration::from_millis(1000));

        map.remove(&1);

        assert!(map.get(&1).is_none());
        assert!(map.get(&2).is_some());
    }

    #[test]
    fn cleanup_expired() {
        let mut map = ExpiringDataHashMap::new();

        map.insert_with_expiry(1, "hello", Duration::from_millis(1000));
        map.insert_with_expiry(2, "world", Duration::from_millis(2));
        map.insert_with_expiry(3, "how", Duration::from_millis(1000));
        map.insert_with_expiry(4, "are", Duration::from_millis(3));
        map.insert_with_expiry(5, "you", Duration::from_millis(1000));

        std::thread::sleep(Duration::from_millis(10));

        map.cleanup_expired();

        assert_eq!(map.data.len(), 3);

        assert!(map.get(&1).is_some());
        assert!(map.get(&2).is_none());
        assert!(map.get(&3).is_some());
        assert!(map.get(&4).is_none());
        assert!(map.get(&5).is_some());
    }
}
