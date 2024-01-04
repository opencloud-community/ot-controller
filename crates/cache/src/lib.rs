// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use core::fmt::Display;
use core::time::Duration;
use moka::future::Cache as LocalCache;
use redis::{AsyncCommands, RedisError, ToRedisArgs};
use serde::de::DeserializeOwned;
use serde::Serialize;
use siphasher::sip128::{Hasher128, SipHasher24};
use std::hash::Hash;
use std::time::Instant;

type RedisConnection = redis::aio::ConnectionManager;

/// Application level cache which can store entries both in a locally and distributed using redis
pub struct Cache<K, V> {
    local: LocalCache<K, LocalEntry<V>>,
    redis: Option<RedisConfig>,
}

struct RedisConfig {
    redis: RedisConnection,
    prefix: String,
    ttl: Duration,
    hash_key: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error(transparent)]
    Redis(#[from] RedisError),
    #[error(transparent)]
    Serde(#[from] bincode::Error),
}

impl<K, V> Cache<K, V>
where
    K: Display + Hash + Eq + Send + Sync + 'static,
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub fn new(ttl: Duration) -> Self {
        Self {
            local: LocalCache::builder().time_to_live(ttl).build(),
            redis: None,
        }
    }

    pub fn with_redis(
        self,
        redis: RedisConnection,
        prefix: impl Into<String>,
        ttl: Duration,
        hash_key: bool,
    ) -> Self {
        Self {
            redis: Some(RedisConfig {
                redis,
                prefix: prefix.into(),
                ttl,
                hash_key,
            }),
            ..self
        }
    }

    /// Return the longest duration an entry might live for
    pub fn longest_ttl(&self) -> Duration {
        let local_ttl = self
            .local
            .policy()
            .time_to_live()
            .expect("local always has a ttl");

        if let Some(redis) = &self.redis {
            redis.ttl.max(local_ttl)
        } else {
            local_ttl
        }
    }

    pub async fn get(&self, key: &K) -> Result<Option<V>, CacheError> {
        if let Some(entry) = self
            .local
            .get(key)
            .await
            .filter(|entry| entry.still_valid())
        {
            Ok(Some(entry.value))
        } else if let Some(RedisConfig {
            redis,
            prefix,
            hash_key,
            ..
        }) = &self.redis
        {
            let v: Option<Vec<u8>> = redis
                .clone()
                .get(RedisCacheKey {
                    prefix,
                    key,
                    hash_key: *hash_key,
                })
                .await?;

            if let Some(v) = v {
                let v = bincode::deserialize(&v)?;

                Ok(Some(v))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Insert a key-value pair with the cache's default TTL
    pub async fn insert(&self, key: K, value: V) -> Result<(), CacheError> {
        if let Some(RedisConfig {
            redis,
            prefix,
            ttl,
            hash_key,
        }) = &self.redis
        {
            redis
                .clone()
                .set_ex(
                    RedisCacheKey {
                        prefix,
                        key: &key,
                        hash_key: *hash_key,
                    },
                    bincode::serialize(&value)?,
                    ttl.as_secs(),
                )
                .await?;
        }

        self.local
            .insert(
                key,
                LocalEntry {
                    value,
                    expires_at: None,
                },
            )
            .await;

        Ok(())
    }

    /// Insert an entry with a custom TTL
    ///
    /// Note that TTLs larger than the configured one will be ignored
    pub async fn insert_with_ttl(&self, key: K, value: V, ttl: Duration) -> Result<(), CacheError> {
        if ttl >= self.longest_ttl() {
            return self.insert(key, value).await;
        }

        if let Some(RedisConfig {
            redis,
            prefix,
            hash_key,
            ..
        }) = &self.redis
        {
            redis
                .clone()
                .set_ex(
                    RedisCacheKey {
                        prefix,
                        key: &key,
                        hash_key: *hash_key,
                    },
                    bincode::serialize(&value)?,
                    ttl.as_secs(),
                )
                .await?;
        }

        self.local
            .insert(
                key,
                LocalEntry {
                    value,
                    expires_at: Some(Instant::now() + ttl),
                },
            )
            .await;

        Ok(())
    }
}

/// [`ToRedisArgs`] implementation for the cache-key
/// Takes the prefix and cache-key to turn them into a redis-key
struct RedisCacheKey<'a, K> {
    hash_key: bool,
    prefix: &'a str,
    key: &'a K,
}

impl<K: Display + Hash> Display for RedisCacheKey<'_, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.hash_key {
            let mut h = SipHasher24::new_with_keys(!0x113, 0x311);
            self.key.hash(&mut h);
            let hash = h.finish128().as_u128();

            write!(f, "opentalk-cache:{}:{:x}", self.prefix, hash)
        } else {
            write!(f, "opentalk-cache:{}:{}", self.prefix, self.key)
        }
    }
}

impl<D: Display + Hash> ToRedisArgs for RedisCacheKey<'_, D> {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        out.write_arg_fmt(self)
    }
}

#[derive(Debug, Clone, Copy)]
struct LocalEntry<V> {
    value: V,
    /// Custom expiration value to work around moka's limitation to set a custom ttl for an entry
    expires_at: Option<Instant>,
}

impl<V> LocalEntry<V> {
    // Check if the custom ttl has expired
    fn still_valid(&self) -> bool {
        if let Some(exp) = self.expires_at {
            exp.saturating_duration_since(Instant::now()) > Duration::ZERO
        } else {
            true
        }
    }
}
