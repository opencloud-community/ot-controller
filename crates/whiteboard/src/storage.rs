// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod redis;

pub(crate) use redis::{del, get, set_initialized, try_start_init};
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, ToRedisArgs, FromRedisValue)]
#[to_redis_args(serde)]
#[from_redis_value(serde)]
pub enum InitState {
    /// Spacedeck is initializing
    Initializing,
    /// Spacedeck has been initialized
    Initialized(SpaceInfo),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceInfo {
    pub id: String,
    pub url: Url,
}
