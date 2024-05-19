// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId, VolatileStaticMemoryStorage};
use opentalk_types::{core::ParticipantId, signaling::media::ParticipantMediaState};
use parking_lot::RwLock;

use super::memory::MemoryMediaState;
use crate::storage::media_storage::MediaStorage;

static STATE: OnceLock<Arc<RwLock<MemoryMediaState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemoryMediaState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl MediaStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_media_state(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<Option<ParticipantMediaState>, SignalingModuleError> {
        Ok(state().read().get_media_state(room, participant))
    }
}
