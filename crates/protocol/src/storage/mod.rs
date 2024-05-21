// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod protocol_storage;
mod redis;
mod volatile;

pub(crate) use protocol_storage::ProtocolStorage;

pub(crate) mod init {
    pub(crate) use super::redis::{
        init_del as del, init_get as get, set_initialized, try_start_init, InitState,
    };
}
pub(crate) mod session {
    pub(crate) use super::redis::{
        session_get as get, session_get_del as get_del, session_set as set,
    };
}
pub(crate) use redis::cleanup;

#[cfg(test)]
mod test_common {
    use opentalk_signaling_core::SignalingRoomId;

    use super::ProtocolStorage;

    pub const ROOM: SignalingRoomId = SignalingRoomId::nil();

    pub const GROUP_ID_A: &str = "group_id A";
    pub const GROUP_ID_B: &str = "group_id B";

    pub(crate) async fn group(storage: &mut dyn ProtocolStorage) {
        storage.group_set(ROOM, GROUP_ID_A).await.unwrap();

        assert_eq!(GROUP_ID_A, storage.group_get(ROOM).await.unwrap().unwrap());

        storage.group_set(ROOM, GROUP_ID_B).await.unwrap();
        assert_eq!(GROUP_ID_B, storage.group_get(ROOM).await.unwrap().unwrap());
    }
}
