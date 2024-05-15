// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use opentalk_types::core::ParticipantId;
use parking_lot::RwLock;

use super::memory::MemoryControlState;
use crate::{
    control::storage::control_storage::ControlStorage, SignalingModuleError, SignalingRoomId,
    VolatileStaticMemoryStorage,
};

static STATE: OnceLock<Arc<RwLock<MemoryControlState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemoryControlState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl ControlStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn participant_set_exists(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<bool, SignalingModuleError> {
        Ok(state().read().participant_set_exists(room))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_all_participants(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Vec<ParticipantId>, SignalingModuleError> {
        Ok(state().read().get_all_participants(room))
    }
}
