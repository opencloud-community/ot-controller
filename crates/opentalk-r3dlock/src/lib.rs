// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Implementation of a redlock mutex for a single redis instance

use std::{
    ops::Range,
    time::{Duration, Instant},
};

use rand::{Rng, rng};
use redis::{RedisError, Script, ToRedisArgs, Value, aio::ConnectionLike};
use snafu::Snafu;
use tokio::time::sleep;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Snafu, Debug, PartialEq)]
pub enum Error {
    #[snafu(display("Failed to unlock because redis returned no success"))]
    FailedToUnlock,
    #[snafu(display("Failed to unlock because the lock already expired in redis"))]
    AlreadyExpired,
    #[snafu(display("Failed to acquire the lock"))]
    CouldNotAcquireLock,
    #[snafu(display("Failed to connect to RabbitMQ: {source}"), context(false))]
    Redis { source: RedisError },
}

const LOCK_TIME: Duration = Duration::from_secs(30);

const UNLOCK_SCRIPT: &str = r"
if redis.call('get',KEYS[1]) == ARGV[1] then
    return redis.call('del',KEYS[1])
else
    return 0
end";

/// Represents a redlock mutex over a resource inside a single redis instance
///
/// The lock can be acquired using [`lock()`](Mutex::lock()).
pub struct Mutex<K> {
    key: K,

    wait_time: Range<Duration>,
    tries: usize,
}

/// Represents a locked redlock mutex
///
/// As these locks can expire in redis, this carries an Instant. Call [`is_locked()`](MutexGuard::is_locked())
/// During unlock, it is checked whether the canary is still present as the locks key value.
pub struct MutexGuard<K> {
    key: K,
    canary: Vec<u8>,
    created: Instant,
    locked: bool,
}

impl<K> MutexGuard<K> {
    /// Returns true when the [`MutexGuard`] / locked redlock mutex is still valid
    ///
    /// If the [`MutexGuard`]/lock expired in Redis, this returns false.
    pub fn is_locked(&self) -> bool {
        self.is_locked_internal() && !self.is_expired()
    }

    fn is_expired(&self) -> bool {
        self.created.elapsed() > LOCK_TIME
    }

    fn is_locked_internal(&self) -> bool {
        self.locked
    }
}

impl<K> MutexGuard<K>
where
    K: ToRedisArgs,
{
    /// Unlocks this [`MutexGuard`] / locked redlock mutex
    ///
    /// If Redis fails to unlock this lock, or this lock is already unlocked, this method returns a [`Error`]
    pub async fn unlock<C>(mut self, redis: &mut C) -> Result<()>
    where
        C: ConnectionLike,
    {
        self.locked = false;
        if self.is_expired() {
            return AlreadyExpiredSnafu.fail();
        }

        let script = Script::new(UNLOCK_SCRIPT);
        let result: i32 = script
            .key(ToRedisArgsRef(&self.key))
            .arg(&self.canary[..])
            .invoke_async(redis)
            .await?;

        if result == 1 {
            Ok(())
        } else {
            FailedToUnlockSnafu.fail()
        }
    }
}

impl<K> Drop for MutexGuard<K> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            debug_assert!(!self.is_locked(), "MutexGuard must be unlocked before drop");
        }
    }
}

impl<K> Mutex<K>
where
    K: ToRedisArgs,
{
    /// Creates a new [`Mutex`]
    ///
    /// Takes a key which represents the resource used as a lock
    pub fn new(key: K) -> Self
    where
        K: ToRedisArgs,
    {
        Self {
            key,
            wait_time: Duration::from_millis(10)..Duration::from_millis(50),
            tries: 10,
        }
    }

    /// Set a duration range to randomly wait between retries
    pub fn with_wait_time(mut self, range: Range<Duration>) -> Self {
        self.wait_time = range;
        self
    }

    /// Set the amount of locking retries
    pub fn with_retries(mut self, retries: usize) -> Self {
        self.tries = retries.saturating_add(1);
        self
    }

    /// Locks the [`Mutex`] and returns a [`MutexGuard`] / redlock mutex
    pub async fn lock<C>(self, redis: &mut C) -> Result<MutexGuard<K>>
    where
        C: ConnectionLike,
    {
        let canary = rng()
            .sample_iter(rand::distr::Alphanumeric)
            .take(20)
            .collect::<Vec<u8>>();

        for _ in 0..self.tries {
            let created = Instant::now();

            // Send the SET command to create a lock with the following args:
            // Key: The lock key
            // Value: Canary which is checked during unlock, to see if this is poised
            // NX: Only set the key if it not exists on the server
            // PX + Time: Set expire time
            let res: redis::Value = redis::cmd("SET")
                .arg(ToRedisArgsRef(&self.key))
                .arg(&canary[..])
                .arg("NX")
                .arg("PX")
                .arg(LOCK_TIME.as_millis() as u64)
                .query_async(redis)
                .await?;

            if let Value::Okay = res {
                let guard = MutexGuard {
                    key: self.key,
                    canary,
                    created,
                    locked: true,
                };
                return Ok(guard);
            } else {
                sleep(rng().random_range(self.wait_time.clone())).await;
            }
        }

        CouldNotAcquireLockSnafu.fail()
    }
}

/// This as a workaround for the missing impl ToRedisArg for &ToRedisArg, to avoid clones and copies
struct ToRedisArgsRef<'k, K>(&'k K);

impl<K> ToRedisArgs for ToRedisArgsRef<'_, K>
where
    K: ToRedisArgs,
{
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        self.0.write_redis_args(out)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn test_lock_unlock_and_relock() {
        let redis_url =
            std::env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://localhost:6379/".to_owned());
        let redis = redis::Client::open(redis_url).expect("Invalid redis url");

        let mut redis_conn = redis
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to get redis connection");

        let mutex = Mutex::new("test-1MY-REDIS-LOCK");

        let guard = mutex.lock(&mut redis_conn).await.unwrap();
        guard.unlock(&mut redis_conn).await.unwrap();

        let mutex = Mutex::new("test-1MY-REDIS-LOCK");
        let guard2 = mutex.lock(&mut redis_conn).await.unwrap();
        guard2.unlock(&mut redis_conn).await.unwrap();
    }

    #[tokio::test]
    async fn test_double_locking() {
        let redis_url =
            std::env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://localhost:6379/".to_owned());
        let redis = redis::Client::open(redis_url).expect("Invalid redis url");

        let mut redis_conn = redis
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to get redis connection");

        let mutex1 = Mutex::new("test2-MY-REDIS-LOCK");
        let mutex2 = Mutex::new("test2-MY-REDIS-LOCK");

        let guard1 = mutex1.lock(&mut redis_conn).await.unwrap();
        let guard2 = mutex2.lock(&mut redis_conn).await.err().unwrap();

        // Unlock first to cleanup redis ressources.
        guard1.unlock(&mut redis_conn).await.unwrap();

        assert_eq!(guard2, Error::CouldNotAcquireLock);
    }
}
