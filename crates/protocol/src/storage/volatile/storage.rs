// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId, VolatileStaticMemoryStorage};
use parking_lot::RwLock;

use super::memory::MemoryProtocolState;
use crate::storage::protocol_storage::ProtocolStorage;

static STATE: OnceLock<Arc<RwLock<MemoryProtocolState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemoryProtocolState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl ProtocolStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(name = "set_protocol_group", skip(self))]
    async fn group_set(
        &mut self,
        room: SignalingRoomId,
        group: &str,
    ) -> Result<(), SignalingModuleError> {
        state().write().group_set(room, group);
        Ok(())
    }
}
