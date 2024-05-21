// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod protocol_storage;
mod redis;
mod volatile;

pub(crate) use protocol_storage::ProtocolStorage;

pub(crate) mod group {
    pub(crate) use super::redis::group_get as get;
}
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
mod test_common {}
