// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    fmt::Debug,
    time::{Duration, SystemTime},
};

use opentalk_types::core::BreakoutRoomId;
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

use super::BreakoutRoom;

mod redis;

pub(crate) use redis::{del_config, get_config, set_config};

/// Configuration of the current breakout rooms which lives inside redis
///
/// When the configuration is set the breakoutrooms are considered active.
/// Breakout rooms with a duration will have the redis entry expire
/// whenever the breakoutrooms expire.
#[derive(Debug, Serialize, Deserialize, Clone, ToRedisArgs, FromRedisValue)]
#[to_redis_args(serde)]
#[from_redis_value(serde)]
pub struct BreakoutConfig {
    pub rooms: Vec<BreakoutRoom>,
    pub started: SystemTime,
    pub duration: Option<Duration>,
}

impl BreakoutConfig {
    pub fn is_valid_id(&self, id: BreakoutRoomId) -> bool {
        self.rooms.iter().any(|room| room.id == id)
    }
}
