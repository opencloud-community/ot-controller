// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::BTreeSet,
    sync::{Arc, OnceLock},
};

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId, VolatileStaticMemoryStorage};
use opentalk_types_signaling_legal_vote::{
    parameters::Parameters, tally::Tally, token::Token, vote::LegalVoteId,
};
use parking_lot::RwLock;

use super::memory::MemoryLegalVoteState;
use crate::{
    error::LegalVoteError,
    storage::{
        LegalVoteAllowTokenStorage, LegalVoteCurrentStorage, LegalVoteHistoryStorage,
        LegalVoteParameterStorage, LegalVoteStorage, VoteScriptResult, VoteStatus,
        legal_vote_storage::{LegalVoteCountStorage, LegalVoteProtocolStorage},
        protocol::v1::{ProtocolEntry, Vote},
    },
};

static STATE: OnceLock<Arc<RwLock<MemoryLegalVoteState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemoryLegalVoteState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl LegalVoteStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(name = "legal_vote_end_current_vote", skip(self, end_entry))]
    async fn end_current_vote(
        &mut self,
        room: SignalingRoomId,
        legal_vote: LegalVoteId,
        end_entry: &ProtocolEntry,
    ) -> Result<bool, SignalingModuleError> {
        Ok(state()
            .write()
            .end_current_vote(room, legal_vote, end_entry.clone()))
    }

    #[tracing::instrument(name = "legal_vote_cleanup_vote", skip(self))]
    async fn cleanup_vote(
        &mut self,
        room: SignalingRoomId,
        legal_vote: LegalVoteId,
    ) -> Result<(), SignalingModuleError> {
        state().write().cleanup_vote(room, legal_vote);
        Ok(())
    }

    #[tracing::instrument(name = "legal_vote_cast_vote", skip(self, vote_event))]
    async fn vote(
        &mut self,
        room: SignalingRoomId,
        legal_vote: LegalVoteId,
        vote_event: Vote,
    ) -> Result<VoteScriptResult, LegalVoteError> {
        state().write().vote(room, legal_vote, vote_event)
    }

    async fn get_vote_status(
        &mut self,
        room: SignalingRoomId,
        legal_vote: LegalVoteId,
    ) -> Result<VoteStatus, SignalingModuleError> {
        Ok(state().read().get_vote_status(room, legal_vote))
    }
}

#[async_trait(?Send)]
impl LegalVoteAllowTokenStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(name = "legal_vote_set_allowed_tokens", skip(self, allowed_tokens))]
    async fn allow_token_set(
        &mut self,
        room: SignalingRoomId,
        legal_vote: LegalVoteId,
        allowed_tokens: Vec<Token>,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .allow_token_set(room, legal_vote, allowed_tokens);
        Ok(())
    }
}

#[async_trait(?Send)]
impl LegalVoteCurrentStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(name = "legal_vote_set_current_vote_id", skip(self))]
    async fn current_vote_set(
        &mut self,
        room: SignalingRoomId,
        new_vote: LegalVoteId,
    ) -> Result<bool, SignalingModuleError> {
        Ok(state().write().current_vote_set(room, new_vote))
    }

    #[tracing::instrument(name = "legal_vote_get_current_vote_id", skip(self))]
    async fn current_vote_get(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<LegalVoteId>, SignalingModuleError> {
        Ok(state().read().current_vote_get(room))
    }

    #[tracing::instrument(name = "legal_vote_delete_current_vote_id", skip(self))]
    async fn current_vote_delete(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        state().write().current_vote_delete(room);
        Ok(())
    }
}

#[async_trait(?Send)]
impl LegalVoteHistoryStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(name = "legal_vote_get_history", skip(self))]
    async fn history_get(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<BTreeSet<LegalVoteId>, SignalingModuleError> {
        Ok(state().read().history_get(room))
    }

    #[tracing::instrument(name = "legal_vote_history_contains", skip(self))]
    async fn history_contains(
        &mut self,
        room: SignalingRoomId,
        vote: LegalVoteId,
    ) -> Result<bool, SignalingModuleError> {
        Ok(state().read().history_contains(room, vote))
    }

    #[tracing::instrument(name = "legal_vote_delete_history", skip(self))]
    async fn history_delete(&mut self, room: SignalingRoomId) -> Result<(), SignalingModuleError> {
        state().write().history_delete(room);
        Ok(())
    }
}

#[async_trait(?Send)]
impl LegalVoteParameterStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(name = "legal_vote_set_parameters", skip(self, parameters))]
    async fn parameter_set(
        &mut self,
        room: SignalingRoomId,
        legal_vote: LegalVoteId,
        parameters: &Parameters,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .parameter_set(room, legal_vote, parameters.clone());
        Ok(())
    }

    #[tracing::instrument(name = "legal_vote_get_parameters", skip(self))]
    async fn parameter_get(
        &mut self,
        room: SignalingRoomId,
        legal_vote: LegalVoteId,
    ) -> Result<Option<Parameters>, SignalingModuleError> {
        Ok(state().read().parameter_get(room, legal_vote))
    }
}

#[async_trait(?Send)]
impl LegalVoteCountStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(name = "legal_vote_get_vote_count", skip(self))]
    async fn count_get(
        &mut self,
        room: SignalingRoomId,
        legal_vote: LegalVoteId,
        enable_abstain: bool,
    ) -> Result<Tally, SignalingModuleError> {
        Ok(state().read().count_get(room, legal_vote, enable_abstain))
    }
}

#[async_trait(?Send)]
impl LegalVoteProtocolStorage for VolatileStaticMemoryStorage {
    /// Add an entry to the vote protocol of `legal_vote_id`
    #[tracing::instrument(name = "legal_vote_add_protocol_entry", skip(self, entry))]
    async fn protocol_add_entry(
        &mut self,
        room: SignalingRoomId,
        legal_vote: LegalVoteId,
        entry: ProtocolEntry,
    ) -> Result<(), SignalingModuleError> {
        state().write().protocol_add_entry(room, legal_vote, entry);
        Ok(())
    }

    /// Get the vote protocol for `legal_vote_id`
    #[tracing::instrument(name = "legal_vote_get_protocol", skip(self))]
    async fn protocol_get(
        &mut self,
        room: SignalingRoomId,
        legal_vote: LegalVoteId,
    ) -> Result<Vec<ProtocolEntry>, SignalingModuleError> {
        Ok(state().read().protocol_get(room, legal_vote))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use opentalk_signaling_core::VolatileStaticMemoryStorage;
    use serial_test::serial;

    use super::state;
    use crate::storage::test_common;

    fn storage() -> VolatileStaticMemoryStorage {
        state().write().reset();
        VolatileStaticMemoryStorage
    }

    #[tokio::test]
    #[serial]
    async fn allow_token() {
        test_common::allow_token(&mut storage()).await
    }

    #[tokio::test]
    #[serial]
    async fn current_vote() {
        test_common::current_vote(&mut storage()).await
    }

    #[tokio::test]
    #[serial]
    async fn parameter() {
        test_common::parameter(&mut storage()).await
    }

    #[tokio::test]
    #[serial]
    async fn voting() {
        test_common::voting(&mut storage()).await
    }
}
