// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeSet, HashMap};

use chrono::Utc;
use opentalk_signaling_core::SignalingRoomId;
use opentalk_types_signaling_legal_vote::{
    parameters::Parameters,
    tally::Tally,
    token::Token,
    vote::{LegalVoteId, VoteOption},
};

use crate::{
    error::{ErrorKind, LegalVoteError},
    storage::{
        protocol::v1::{ProtocolEntry, Vote, VoteEvent},
        VoteScriptResult, VoteStatus,
    },
};

#[derive(Debug, Clone, Default)]
pub(crate) struct MemoryLegalVoteState {
    allowed_tokens: HashMap<(SignalingRoomId, LegalVoteId), BTreeSet<Token>>,
    count: HashMap<(SignalingRoomId, LegalVoteId), Tally>,
    parameters: HashMap<(SignalingRoomId, LegalVoteId), Parameters>,
    protocol: HashMap<(SignalingRoomId, LegalVoteId), Vec<ProtocolEntry>>,
    current_vote: HashMap<SignalingRoomId, LegalVoteId>,
    history: HashMap<SignalingRoomId, BTreeSet<LegalVoteId>>,
}

impl MemoryLegalVoteState {
    #[cfg(test)]
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn end_current_vote(
        &mut self,
        room: SignalingRoomId,
        vote: LegalVoteId,
        end_entry: ProtocolEntry,
    ) -> bool {
        if Some(vote) != self.current_vote_get(room) {
            return false;
        }

        self.current_vote_delete(room);
        self.protocol_add_entry(room, vote, end_entry);
        self.history.entry(room).or_default().insert(vote);
        true
    }

    pub(crate) fn cleanup_vote(&mut self, room: SignalingRoomId, legal_vote: LegalVoteId) {
        if Some(legal_vote) == self.current_vote_get(room) {
            self.current_vote_delete(room)
        }

        self.parameters.remove(&(room, legal_vote));
        self.allowed_tokens.remove(&(room, legal_vote));
        self.protocol.remove(&(room, legal_vote));
        self.count.remove(&(room, legal_vote));
    }

    pub(crate) fn vote(
        &mut self,
        room: SignalingRoomId,
        vote: LegalVoteId,
        vote_event: Vote,
    ) -> Result<VoteScriptResult, LegalVoteError> {
        let vote_options = vote_event.option;
        let vote_token = vote_event.token;

        let parameters = self.parameter_get(room, vote).ok_or(LegalVoteError::Vote {
            source: ErrorKind::InvalidVoteId,
        })?;
        let timestamp = (!parameters.inner.kind.is_hidden()).then(Utc::now);
        let entry = ProtocolEntry::new_with_optional_time(timestamp, VoteEvent::Vote(vote_event));
        if self
            .current_vote_get(room)
            .is_none_or(|current_vote| current_vote != vote)
        {
            return Ok(VoteScriptResult::InvalidVoteId);
        }
        if !self.consume_allow_token(room, vote, vote_token) {
            return Ok(VoteScriptResult::Ineligible);
        }
        self.protocol_add_entry(room, vote, entry);

        let tally = self.count.entry((room, vote)).or_default();
        match vote_options {
            VoteOption::Yes => tally.yes += 1,
            VoteOption::No => tally.no += 1,
            VoteOption::Abstain => *tally.abstain.get_or_insert(0) += 1,
        }

        if self
            .allowed_tokens
            .get(&(room, vote))
            .is_none_or(|set| set.is_empty())
        {
            Ok(VoteScriptResult::SuccessAutoClose)
        } else {
            Ok(VoteScriptResult::Success)
        }
    }

    pub(crate) fn get_vote_status(&self, room: SignalingRoomId, legal: LegalVoteId) -> VoteStatus {
        if self.current_vote_get(room) == Some(legal) {
            VoteStatus::Active
        } else if self.history_contains(room, legal) {
            VoteStatus::Complete
        } else {
            VoteStatus::Unknown
        }
    }

    pub(crate) fn allow_token_set(
        &mut self,
        room: SignalingRoomId,
        vote: LegalVoteId,
        allowed_tokens: Vec<Token>,
    ) {
        self.allowed_tokens
            .insert((room, vote), BTreeSet::from_iter(allowed_tokens));
    }

    pub(crate) fn current_vote_set(
        &mut self,
        room: SignalingRoomId,
        new_vote: LegalVoteId,
    ) -> bool {
        match self.current_vote.entry(room) {
            std::collections::hash_map::Entry::Occupied(_) => false,
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(new_vote);
                true
            }
        }
    }

    pub(crate) fn current_vote_get(&self, room: SignalingRoomId) -> Option<LegalVoteId> {
        self.current_vote.get(&room).copied()
    }

    pub(crate) fn current_vote_delete(&mut self, room: SignalingRoomId) {
        self.current_vote.remove(&room);
    }

    pub(crate) fn history_get(&self, room: SignalingRoomId) -> BTreeSet<LegalVoteId> {
        self.history.get(&room).cloned().unwrap_or_default()
    }

    pub(crate) fn history_contains(&self, room: SignalingRoomId, vote: LegalVoteId) -> bool {
        self.history
            .get(&room)
            .is_some_and(|history| history.contains(&vote))
    }

    pub(crate) fn history_delete(&mut self, room: SignalingRoomId) {
        self.history.remove(&room);
    }

    pub(crate) fn parameter_set(
        &mut self,
        room: SignalingRoomId,
        vote: LegalVoteId,
        parameters: Parameters,
    ) {
        self.parameters.insert((room, vote), parameters);
    }

    pub(crate) fn parameter_get(
        &self,
        room: SignalingRoomId,
        vote: LegalVoteId,
    ) -> Option<Parameters> {
        self.parameters.get(&(room, vote)).cloned()
    }

    pub(crate) fn count_get(
        &self,
        room: SignalingRoomId,
        vote: LegalVoteId,
        enable_abstain: bool,
    ) -> Tally {
        let mut tally = self.count.get(&(room, vote)).cloned().unwrap_or_default();
        if enable_abstain {
            tally.abstain.get_or_insert(0);
        } else {
            tally.abstain.take();
        }
        tally
    }

    pub(crate) fn protocol_add_entry(
        &mut self,
        room: SignalingRoomId,
        vote: LegalVoteId,
        entry: ProtocolEntry,
    ) {
        self.protocol.entry((room, vote)).or_default().push(entry);
    }

    pub(crate) fn protocol_get(
        &self,
        room: SignalingRoomId,
        vote: LegalVoteId,
    ) -> Vec<ProtocolEntry> {
        self.protocol
            .get(&(room, vote))
            .cloned()
            .unwrap_or_default()
    }

    fn consume_allow_token(
        &mut self,
        room: SignalingRoomId,
        vote: LegalVoteId,
        token: Token,
    ) -> bool {
        self.allowed_tokens
            .get_mut(&(room, vote))
            .is_some_and(|tokens| tokens.remove(&token))
    }
}
