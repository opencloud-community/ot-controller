// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_types::core::{ResumptionToken, TicketToken};

use super::SignalingStorageError;
use crate::api::signaling::{resumption::ResumptionData, ticket::TicketData};

#[async_trait(?Send)]
pub(crate) trait SignalingStorage {
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
}
