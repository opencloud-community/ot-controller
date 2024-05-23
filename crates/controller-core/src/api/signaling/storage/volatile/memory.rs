// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use opentalk_signaling_core::ExpiringDataHashMap;
use opentalk_types::core::TicketToken;

use crate::api::signaling::{storage::TICKET_EXPIRY_SECONDS, ticket::TicketData};

#[derive(Debug, Clone, Default)]
pub(super) struct MemorySignalingState {
    tickets: ExpiringDataHashMap<TicketToken, TicketData>,
}

impl MemorySignalingState {
    #[cfg(test)]
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn set_ticket_ex(&mut self, ticket_token: TicketToken, ticket_data: TicketData) {
        self.tickets.insert_with_expiry(
            ticket_token,
            ticket_data,
            Duration::from_secs(TICKET_EXPIRY_SECONDS),
        );
    }
}
