// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, OnceLock},
};

use async_trait::async_trait;
use opentalk_signaling_core::{
    NotFoundSnafu, SignalingModuleError, SignalingRoomId, VolatileStaticMemoryStorage,
};
use opentalk_types_signaling_polls::{state::PollsState, ChoiceId, PollId};
use parking_lot::RwLock;
use snafu::OptionExt;

use super::memory::MemoryPollsState;
use crate::storage::polls_storage::PollsStorage;

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

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_polls_state(
        &mut self,
        room: SignalingRoomId,
        polls_state: &PollsState,
    ) -> Result<bool, SignalingModuleError> {
        Ok(state().write().set_polls_state(room, polls_state))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_polls_state(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        state().write().delete_polls_state(&room);
        Ok(())
    }

    async fn delete_poll_results(
        &mut self,
        room: SignalingRoomId,
        poll_id: PollId,
    ) -> Result<(), SignalingModuleError> {
        state().write().delete_polls_results(room, poll_id);
        Ok(())
    }

    async fn results(
        &mut self,
        room: SignalingRoomId,
        poll: PollId,
    ) -> Result<BTreeMap<ChoiceId, u32>, SignalingModuleError> {
        state()
            .read()
            .poll_results(room, poll)
            .with_context(|| NotFoundSnafu {
                message: format!("Could not find results for poll {poll} in room {room}"),
            })
    }

    async fn vote(
        &mut self,
        room: SignalingRoomId,
        poll_id: PollId,
        previous_choice_ids: &BTreeSet<ChoiceId>,
        new_choice_ids: &BTreeSet<ChoiceId>,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .vote(room, poll_id, previous_choice_ids, new_choice_ids)
    }

    async fn add_poll_to_list(
        &mut self,
        room: SignalingRoomId,
        poll_id: PollId,
    ) -> Result<(), SignalingModuleError> {
        state().write().add_poll_to_list(room, poll_id);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn poll_ids(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Vec<PollId>, SignalingModuleError> {
        Ok(state().read().poll_ids(room))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_poll_ids(&mut self, room: SignalingRoomId) -> Result<(), SignalingModuleError> {
        state().write().delete_poll_ids(room);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use opentalk_signaling_core::VolatileStaticMemoryStorage;
    use serial_test::serial;

    use super::{super::super::test_common, state};

    async fn storage() -> VolatileStaticMemoryStorage {
        state().write().reset();
        VolatileStaticMemoryStorage
    }

    #[tokio::test]
    #[serial]
    async fn polls_state() {
        test_common::polls_state(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn voting() {
        test_common::voting(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn polls() {
        test_common::polls(&mut storage().await).await;
    }
}
