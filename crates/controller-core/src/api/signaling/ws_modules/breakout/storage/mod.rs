// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    fmt::Debug,
    time::{Duration, SystemTime},
};

use opentalk_types_common::rooms::BreakoutRoomId;
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

use super::BreakoutRoom;

mod breakout_storage;
mod redis;
mod volatile;

pub use breakout_storage::BreakoutStorage;

/// Configuration of the current breakout rooms which lives inside redis
///
/// When the configuration is set the breakoutrooms are considered active.
/// Breakout rooms with a duration will have the redis entry expire
/// whenever the breakoutrooms expire.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToRedisArgs, FromRedisValue)]
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

#[cfg(test)]
mod test_common {
    use std::time::{Duration, SystemTime};

    use opentalk_types::core::RoomId;
    use pretty_assertions::assert_eq;

    use super::BreakoutStorage;
    use crate::api::signaling::breakout::storage::BreakoutConfig;

    pub const ROOM: RoomId = RoomId::nil();

    pub(super) async fn config_unlimited(storage: &mut impl BreakoutStorage) {
        assert!(storage.get_breakout_config(ROOM).await.unwrap().is_none());

        let config = BreakoutConfig {
            rooms: Vec::new(),
            started: SystemTime::now(),
            duration: None,
        };

        assert!(storage
            .set_breakout_config(ROOM, &config)
            .await
            .unwrap()
            .is_none());

        assert_eq!(
            storage.get_breakout_config(ROOM).await.unwrap(),
            Some(config)
        );

        storage.del_breakout_config(ROOM).await.unwrap();

        assert!(storage.get_breakout_config(ROOM).await.unwrap().is_none());
    }

    pub(super) async fn config_expiring(storage: &mut impl BreakoutStorage) {
        assert!(storage.get_breakout_config(ROOM).await.unwrap().is_none());

        let config = BreakoutConfig {
            rooms: Vec::new(),
            started: SystemTime::now(),
            duration: Some(Duration::from_millis(3)),
        };

        let real_duration = storage
            .set_breakout_config(ROOM, &config)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            storage.get_breakout_config(ROOM).await.unwrap(),
            Some(config)
        );

        std::thread::sleep(real_duration.saturating_add(Duration::from_millis(100)));

        assert!(storage.get_breakout_config(ROOM).await.unwrap().is_none());
    }
}
