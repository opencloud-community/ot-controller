// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use async_trait::async_trait;
use opentalk_signaling_core::{
    control::storage::{ControlStorageParticipantAttributesRaw, ControlStorageParticipantSet},
    SignalingModuleError, SignalingRoomId,
};
use opentalk_types_signaling_legal_vote::{
    parameters::Parameters, tally::Tally, token::Token, vote::LegalVoteId,
};

use super::{
    protocol::v1::{ProtocolEntry, Vote},
    VoteScriptResult, VoteStatus,
};
use crate::error::LegalVoteError;

#[async_trait(?Send)]
pub(crate) trait LegalVoteStorage:
    LegalVoteAllowTokenStorage
    + LegalVoteCurrentStorage
    + LegalVoteHistoryStorage
    + LegalVoteParameterStorage
    + LegalVoteProtocolStorage
    + LegalVoteCountStorage
    + ControlStorageParticipantSet
    + ControlStorageParticipantAttributesRaw
{
    /// End the current vote by moving the vote id to the history & adding a stop/cancel entry
    /// to the vote protocol. See [`END_CURRENT_VOTE_SCRIPT`] for details.
    ///
    /// #Returns
    /// `Ok(true)` when the legal_vote was successfully moved to the history
    /// `Ok(false)` when there is no current vote active
    /// `Err(anyhow::Error)` when a redis error occurred
    async fn end_current_vote(
        &mut self,
        room: SignalingRoomId,
        legal_vote: LegalVoteId,
        end_entry: &ProtocolEntry,
    ) -> Result<bool, SignalingModuleError>;

    /// Cleanup redis keys related to a vote
    ///
    /// See [`CLEANUP_SCRIPT`] for details.
    ///
    /// Deletes all entries associated with the room & vote id.
    async fn cleanup_vote(
        &mut self,
        room: SignalingRoomId,
        legal_vote: LegalVoteId,
    ) -> Result<(), SignalingModuleError>;

    /// Cast a vote for the specified option
    ///
    /// The vote is done atomically on redis with a Lua script.
    /// See [`VOTE_SCRIPT`] for more details.
    async fn vote(
        &mut self,
        room: SignalingRoomId,
        legal_vote: LegalVoteId,
        vote_event: Vote,
    ) -> Result<VoteScriptResult, LegalVoteError>;

    async fn get_vote_status(
        &mut self,
        room: SignalingRoomId,
        legal_vote: LegalVoteId,
    ) -> Result<VoteStatus, SignalingModuleError>;
}

#[async_trait(?Send)]
pub(crate) trait LegalVoteAllowTokenStorage {
    /// Set the list of allowed tokens for the provided `legal_vote`
    async fn allow_token_set(
        &mut self,
        room: SignalingRoomId,
        legal_vote: LegalVoteId,
        allowed_tokens: Vec<Token>,
    ) -> Result<(), SignalingModuleError>;
}

#[async_trait(?Send)]
pub(crate) trait LegalVoteCurrentStorage {
    /// Set the current vote id to `new_vote`
    ///
    /// Set the current vote id only if the key does not exist yet.
    ///
    /// # Returns
    /// - `Ok(true)` when the key got set.
    /// - `Ok(false)` when the key already exists and no changes were made.
    /// - `Err(anyhow::Error)` when a redis error occurred.
    async fn current_vote_set(
        &mut self,
        room: SignalingRoomId,
        new_vote: LegalVoteId,
    ) -> Result<bool, SignalingModuleError>;

    /// Get the currently active vote id
    async fn current_vote_get(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<LegalVoteId>, SignalingModuleError>;

    /// Delete the current vote id key
    async fn current_vote_delete(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError>;
}

#[async_trait(?Send)]
pub(crate) trait LegalVoteHistoryStorage {
    /// Get the vote history as a hashset
    async fn history_get(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<BTreeSet<LegalVoteId>, SignalingModuleError>;

    async fn history_contains(
        &mut self,
        room: SignalingRoomId,
        vote: LegalVoteId,
    ) -> Result<bool, SignalingModuleError>;

    /// Delete the vote history key
    async fn history_delete(&mut self, room: SignalingRoomId) -> Result<(), SignalingModuleError>;
}

#[async_trait(?Send)]
pub(crate) trait LegalVoteParameterStorage {
    /// Set the vote [`Parameters`] for the provided `legal_vote`
    async fn parameter_set(
        &mut self,
        room: SignalingRoomId,
        legal_vote: LegalVoteId,
        parameters: &Parameters,
    ) -> Result<(), SignalingModuleError>;

    /// Get the [`Parameters`] for the provided `legal_vote`
    async fn parameter_get(
        &mut self,
        room: SignalingRoomId,
        legal_vote: LegalVoteId,
    ) -> Result<Option<Parameters>, SignalingModuleError>;
}

#[async_trait(?Send)]
pub(crate) trait LegalVoteCountStorage {
    /// Get the vote count for the specified `legal_vote`
    async fn count_get(
        &mut self,
        room: SignalingRoomId,
        legal_vote: LegalVoteId,
        enable_abstain: bool,
    ) -> Result<Tally, SignalingModuleError>;
}

#[async_trait(?Send)]
pub(crate) trait LegalVoteProtocolStorage {
    /// Add an entry to the vote protocol of `legal_vote`
    async fn protocol_add_entry(
        &mut self,
        room: SignalingRoomId,
        legal_vote: LegalVoteId,
        entry: ProtocolEntry,
    ) -> Result<(), SignalingModuleError>;

    /// Get the vote protocol for `legal_vote`
    async fn protocol_get(
        &mut self,
        room: SignalingRoomId,
        legal_vote: LegalVoteId,
    ) -> Result<Vec<ProtocolEntry>, SignalingModuleError>;
}
