// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod protocol_storage;
mod redis;
mod volatile;

pub(crate) use protocol_storage::ProtocolStorage;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToRedisArgs, FromRedisValue,
)]
#[to_redis_args(serde)]
#[from_redis_value(serde)]
pub enum InitState {
    Initializing,
    Initialized,
}

pub(crate) mod session {
    pub(crate) use super::redis::{
        session_get as get, session_get_del as get_del, session_set as set,
    };
}
pub(crate) use redis::cleanup;
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod test_common {
    use opentalk_signaling_core::SignalingRoomId;

    use super::{InitState, ProtocolStorage};

    pub const ROOM: SignalingRoomId = SignalingRoomId::nil();

    pub const GROUP_ID_A: &str = "group_id A";
    pub const GROUP_ID_B: &str = "group_id B";

    pub(crate) async fn group(storage: &mut dyn ProtocolStorage) {
        storage.group_set(ROOM, GROUP_ID_A).await.unwrap();

        assert_eq!(
            Some(GROUP_ID_A),
            storage.group_get(ROOM).await.unwrap().as_deref()
        );

        storage.group_set(ROOM, GROUP_ID_B).await.unwrap();
        assert_eq!(
            Some(GROUP_ID_B),
            storage.group_get(ROOM).await.unwrap().as_deref()
        );

        storage.group_delete(ROOM).await.unwrap();
        assert_eq!(None, storage.group_get(ROOM).await.unwrap());
    }

    pub(crate) async fn init(storage: &mut dyn ProtocolStorage) {
        assert_eq!(None, storage.try_start_init(ROOM).await.unwrap());
        assert_eq!(
            Some(InitState::Initializing),
            storage.try_start_init(ROOM).await.unwrap()
        );
        storage.set_initialized(ROOM).await.unwrap();
        assert_eq!(
            Some(InitState::Initialized),
            storage.try_start_init(ROOM).await.unwrap()
        );

        assert_eq!(
            Some(InitState::Initialized),
            storage.init_get(ROOM).await.unwrap()
        );

        storage.init_delete(ROOM).await.unwrap();

        assert_eq!(None, storage.init_get(ROOM).await.unwrap());
    }
}
