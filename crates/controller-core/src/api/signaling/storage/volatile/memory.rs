// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{btree_map::Entry, BTreeMap};

use opentalk_signaling_core::{ExpiringDataHashMap, RunnerId};
use opentalk_types::core::TicketToken;
use opentalk_types_common::auth::ResumptionToken;
use opentalk_types_signaling::ParticipantId;

use crate::api::signaling::{
    resumption::ResumptionData,
    storage::{RESUMPTION_TOKEN_EXPIRY, TICKET_EXPIRY},
    ticket::TicketData,
};

#[derive(Debug, Clone, Default)]
pub(super) struct MemorySignalingState {
    tickets: ExpiringDataHashMap<TicketToken, TicketData>,
    resumption_data: ExpiringDataHashMap<ResumptionToken, ResumptionData>,
    participant_runner_locks: BTreeMap<ParticipantId, RunnerId>,
}

impl MemorySignalingState {
    #[cfg(test)]
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn set_ticket_ex(&mut self, ticket_token: TicketToken, ticket_data: TicketData) {
        self.tickets
            .insert_with_expiry(ticket_token, ticket_data, TICKET_EXPIRY);
    }

    pub(super) fn take_ticket(&mut self, ticket_token: &TicketToken) -> Option<TicketData> {
        self.tickets.remove(ticket_token)
    }

    pub(super) fn get_resumption_token_data(
        &self,
        resumption_token: &ResumptionToken,
    ) -> Option<ResumptionData> {
        self.resumption_data.get(resumption_token).cloned()
    }

    pub(super) fn set_resumption_token_data_if_not_exists(
        &mut self,
        resumption_token: ResumptionToken,
        data: ResumptionData,
    ) {
        self.resumption_data.insert_with_expiry_if_not_exists(
            resumption_token,
            data,
            RESUMPTION_TOKEN_EXPIRY,
        );
    }

    pub(super) fn refresh_resumption_token(&mut self, resumption_token: &ResumptionToken) -> bool {
        self.resumption_data
            .update_expiry(resumption_token, RESUMPTION_TOKEN_EXPIRY)
    }

    pub(super) fn delete_resumption_token(&mut self, resumption_token: &ResumptionToken) -> bool {
        self.resumption_data.remove(resumption_token).is_some()
    }

    pub(super) fn try_acquire_participant_id(
        &mut self,
        participant_id: ParticipantId,
        runner_id: RunnerId,
    ) -> bool {
        match self.participant_runner_locks.entry(participant_id) {
            Entry::Vacant(v) => {
                v.insert(runner_id);
                true
            }
            Entry::Occupied(_) => false,
        }
    }

    pub(super) fn participant_id_in_use(&self, participant: ParticipantId) -> bool {
        self.participant_runner_locks.contains_key(&participant)
    }

    pub(super) fn release_participant_id(
        &mut self,
        participant_id: ParticipantId,
    ) -> Option<RunnerId> {
        self.participant_runner_locks.remove(&participant_id)
    }
}
