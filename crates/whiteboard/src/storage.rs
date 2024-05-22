// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod redis;
mod volatile;
mod whiteboard_storage;

pub(crate) use redis::{del, get, set_initialized};
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use url::Url;
pub(crate) use whiteboard_storage::WhiteboardStorage;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToRedisArgs, FromRedisValue)]
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

#[cfg(test)]
mod test_common {
    use opentalk_signaling_core::SignalingRoomId;
    use pretty_assertions::assert_eq;

    use crate::storage::InitState;

    use super::WhiteboardStorage;

    const ROOM: SignalingRoomId = SignalingRoomId::nil();

    pub(super) async fn initialization(storage: &mut dyn WhiteboardStorage) {
        assert!(storage.try_start_init(ROOM).await.unwrap().is_none());
        assert_eq!(
            Some(InitState::Initializing),
            storage.try_start_init(ROOM).await.unwrap()
        );
    }
}
