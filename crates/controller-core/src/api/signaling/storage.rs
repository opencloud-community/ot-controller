// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod error;
mod redis;
mod signaling_storage;
mod volatile;

pub(crate) use error::SignalingStorageError;
pub(crate) use redis::{
    delete_resumption_token, get_resumption_token_data, get_ticket, refresh_resumption_token,
    set_resumption_token_data_if_not_exists,
};
pub(crate) use signaling_storage::SignalingStorage;

const TICKET_EXPIRY_SECONDS: u64 = 30;

#[cfg(test)]
mod test_common {
    use opentalk_signaling_core::Participant;
    use opentalk_types::core::{ParticipantId, ResumptionToken, RoomId, TicketToken};

    use super::SignalingStorage;
    use crate::api::signaling::ticket::TicketData;

    const ALICE: ParticipantId = ParticipantId::from_u128(0xa11c3);

    pub(super) async fn ticket_token(storage: &mut dyn SignalingStorage) {
        let ticket_token = TicketToken::generate();
        let ticket_data = TicketData {
            participant_id: ALICE,
            resuming: false,
            participant: Participant::Guest,
            room: RoomId::generate(),
            breakout_room: None,
            resumption: ResumptionToken::generate(),
        };

        storage
            .set_ticket_ex(&ticket_token, &ticket_data)
            .await
            .unwrap();
    }
}
