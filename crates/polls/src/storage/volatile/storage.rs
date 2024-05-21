// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId, VolatileStaticMemoryStorage};
use opentalk_types::signaling::polls::state::PollsState;
use parking_lot::RwLock;

use crate::storage::polls_storage::PollsStorage;

use super::memory::MemoryPollsState;

static STATE: OnceLock<Arc<RwLock<MemoryPollsState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemoryPollsState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl PollsStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_polls_state(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<PollsState>, SignalingModuleError> {
        Ok(state().read().get_polls_state(room))
    }
}
