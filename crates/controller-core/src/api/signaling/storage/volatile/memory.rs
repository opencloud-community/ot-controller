// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::ExpiringDataHashMap;
use opentalk_types::core::{ResumptionToken, TicketToken};

use crate::api::signaling::{
    resumption::ResumptionData,
    storage::{RESUMPTION_TOKEN_EXPIRY, TICKET_EXPIRY},
    ticket::TicketData,
};

#[derive(Debug, Clone, Default)]
pub(super) struct MemorySignalingState {
    tickets: ExpiringDataHashMap<TicketToken, TicketData>,
    resumption_data: ExpiringDataHashMap<ResumptionToken, ResumptionData>,
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
}
