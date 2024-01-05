// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{ops::Add, time::SystemTime};

use chrono::{DateTime, Utc};
use derive_more::{AsRef, Deref, Display, From, FromStr};

#[allow(unused_imports)]
use crate::imports::*;

/// A UTC DateTime wrapper that implements ToRedisArgs and FromRedisValue.
///
/// The values are stores as unix timestamps in redis.
#[derive(
    AsRef, Deref, Display, From, FromStr, Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash,
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    /// Create a timestamp with the date of the unix epoch start
    /// (1970-01-01 00:00:00 UTC)
    pub fn unix_epoch() -> Self {
        Self(chrono::DateTime::from(std::time::UNIX_EPOCH))
    }

    /// Create a timestamp with the current system time
    pub fn now() -> Timestamp {
        Timestamp(Utc::now())
    }
}

impl From<SystemTime> for Timestamp {
    fn from(value: SystemTime) -> Self {
        Self(value.into())
    }
}

impl From<Timestamp> for DateTime<Utc> {
    fn from(value: Timestamp) -> Self {
        value.0
    }
}

impl Add<chrono::Duration> for Timestamp {
    type Output = Timestamp;

    fn add(self, rhs: chrono::Duration) -> Self::Output {
        Timestamp(self.0 + rhs)
    }
}

#[cfg(feature = "redis")]
impl ToRedisArgs for Timestamp {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        self.0.timestamp().write_redis_args(out)
    }

    fn describe_numeric_behavior(&self) -> redis::NumericBehavior {
        redis::NumericBehavior::NumberIsInteger
    }
}

#[cfg(feature = "redis")]
impl FromRedisValue for Timestamp {
    fn from_redis_value(v: &redis::Value) -> RedisResult<Timestamp> {
        use chrono::TimeZone as _;
        let timestamp = Utc
            .timestamp_opt(i64::from_redis_value(v)?, 0)
            .latest()
            .unwrap();
        Ok(Timestamp(timestamp))
    }
}
