// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod error;
mod redis;
mod signaling_storage;
mod volatile;

pub(crate) use error::SignalingStorageError;
pub(crate) use signaling_storage::SignalingStorage;

const TICKET_EXPIRY_SECONDS: u64 = 30;
const RESUMPTION_TOKEN_EXPIRY_SECONDS: u32 = 120;

#[cfg(test)]
mod test_common {
    use opentalk_signaling_core::Participant;
    use opentalk_types::core::{ParticipantId, ResumptionToken, RoomId, TicketToken};
    use pretty_assertions::assert_eq;

    use super::SignalingStorage;
    use crate::api::signaling::{
        resumption::ResumptionData, storage::SignalingStorageError, ticket::TicketData,
    };

    const ALICE: ParticipantId = ParticipantId::from_u128(0xa11c3);
    const BOB: ParticipantId = ParticipantId::from_u128(0x808);

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

        assert_eq!(
            storage.take_ticket(&ticket_token).await.unwrap(),
            Some(ticket_data)
        );
        // Ensure that the previous `take_ticket(…)` call removed the ticket
        assert!(storage.take_ticket(&ticket_token).await.unwrap().is_none(),);
    }

    pub(super) async fn resumption_token(storage: &mut dyn SignalingStorage) {
        let resumption_token = ResumptionToken::generate();
        let resumption_data_1 = ResumptionData {
            participant_id: ALICE,
            participant: Participant::Guest,
            room: RoomId::generate(),
            breakout_room: None,
        };
        let resumption_data_2 = ResumptionData {
            participant_id: BOB,
            participant: Participant::Guest,
            room: RoomId::generate(),
            breakout_room: None,
        };

        assert!(storage
            .get_resumption_token_data(&resumption_token)
            .await
            .unwrap()
            .is_none());

        assert!(matches!(
            storage.refresh_resumption_token(&resumption_token).await,
            Err(SignalingStorageError::ResumptionTokenAlreadyUsed)
        ));

        storage
            .set_resumption_token_data_if_not_exists(&resumption_token, &resumption_data_1)
            .await
            .unwrap();
        assert_eq!(
            storage
                .get_resumption_token_data(&resumption_token)
                .await
                .unwrap()
                .as_ref(),
            Some(&resumption_data_1)
        );
        assert!(storage
            .refresh_resumption_token(&resumption_token)
            .await
            .is_ok(),);

        storage
            .set_resumption_token_data_if_not_exists(&resumption_token, &resumption_data_2)
            .await
            .unwrap();
        assert_eq!(
            storage
                .get_resumption_token_data(&resumption_token)
                .await
                .unwrap()
                .as_ref(),
            Some(&resumption_data_1)
        );

        assert!(storage
            .delete_resumption_token(&resumption_token)
            .await
            .unwrap());
        assert!(!storage
            .delete_resumption_token(&resumption_token)
            .await
            .unwrap());
    }
}
