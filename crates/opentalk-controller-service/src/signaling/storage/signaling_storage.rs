// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use async_trait::async_trait;
use opentalk_signaling_core::RunnerId;
use opentalk_types_common::auth::{ResumptionToken, TicketToken};
use opentalk_types_signaling::ParticipantId;
use snafu::whatever;
use tokio::time::sleep;

use super::SignalingStorageError;
use crate::signaling::{resumption::ResumptionData, ticket::TicketData};

#[async_trait(?Send)]
pub trait SignalingStorage {
    async fn set_ticket_ex(
        &mut self,
        ticket_token: &TicketToken,
        ticket_data: &TicketData,
    ) -> Result<(), SignalingStorageError>;

    async fn take_ticket(
        &mut self,
        ticket_token: &TicketToken,
    ) -> Result<Option<TicketData>, SignalingStorageError>;

    async fn get_resumption_token_data(
        &mut self,
        resumption_token: &ResumptionToken,
    ) -> Result<Option<ResumptionData>, SignalingStorageError>;

    async fn set_resumption_token_data_if_not_exists(
        &mut self,
        resumption_token: &ResumptionToken,
        data: &ResumptionData,
    ) -> Result<(), SignalingStorageError>;

    async fn refresh_resumption_token(
        &mut self,
        resumption_token: &ResumptionToken,
    ) -> Result<(), SignalingStorageError>;

    async fn delete_resumption_token(
        &mut self,
        resumption_token: &ResumptionToken,
    ) -> Result<bool, SignalingStorageError>;

    /// Attempt to acquire a participant id.
    ///
    /// This function will not wait for the lock to become available, therefore
    /// return immediately.
    ///
    /// Returns `Ok(true)` if the lock has been acquired, `Ok(false)` if the lock
    /// is currently held by other code.
    async fn try_acquire_participant_id(
        &mut self,
        participant_id: ParticipantId,
        runner_id: RunnerId,
    ) -> Result<bool, SignalingStorageError>;

    async fn acquire_participant_id(
        &mut self,
        participant_id: ParticipantId,
        runner_id: RunnerId,
    ) -> Result<(), SignalingStorageError> {
        // Try for up to 10 secs to acquire the key
        for _ in 0..10 {
            if self
                .try_acquire_participant_id(participant_id, runner_id)
                .await?
            {
                return Ok(());
            }
            sleep(Duration::from_secs(1)).await;
        }

        whatever!("Failed to acquire runner id");
    }

    async fn participant_id_in_use(
        &mut self,
        participant_id: ParticipantId,
    ) -> Result<bool, SignalingStorageError>;

    /// Releases the participant_id and returns the id of the runner which held the lock.
    ///
    /// Returns None if the participant id was not locked.
    async fn release_participant_id(
        &mut self,
        participant_id: ParticipantId,
    ) -> Result<Option<RunnerId>, SignalingStorageError>;
}
