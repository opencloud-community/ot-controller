// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use opentalk_signaling_core::{RunnerId, VolatileStaticMemoryStorage};
use opentalk_types_common::auth::{ResumptionToken, TicketToken};
use opentalk_types_signaling::ParticipantId;
use parking_lot::RwLock;
use snafu::ensure;

use super::memory::MemorySignalingState;
use crate::signaling::{
    resumption::ResumptionData,
    storage::{SignalingStorage, SignalingStorageError, error::ResumptionTokenAlreadyUsedSnafu},
    ticket::TicketData,
};

static STATE: OnceLock<Arc<RwLock<MemorySignalingState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemorySignalingState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl SignalingStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_ticket_ex(
        &mut self,
        ticket_token: &TicketToken,
        ticket_data: &TicketData,
    ) -> Result<(), SignalingStorageError> {
        state()
            .write()
            .set_ticket_ex(ticket_token.clone(), ticket_data.clone());
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn take_ticket(
        &mut self,
        ticket_token: &TicketToken,
    ) -> Result<Option<TicketData>, SignalingStorageError> {
        Ok(state().write().take_ticket(ticket_token))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_resumption_token_data(
        &mut self,
        resumption_token: &ResumptionToken,
    ) -> Result<Option<ResumptionData>, SignalingStorageError> {
        Ok(state().read().get_resumption_token_data(resumption_token))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_resumption_token_data_if_not_exists(
        &mut self,
        resumption_token: &ResumptionToken,
        data: &ResumptionData,
    ) -> Result<(), SignalingStorageError> {
        state()
            .write()
            .set_resumption_token_data_if_not_exists(resumption_token.clone(), data.clone());
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn refresh_resumption_token(
        &mut self,
        resumption_token: &ResumptionToken,
    ) -> Result<(), SignalingStorageError> {
        ensure!(
            state().write().refresh_resumption_token(resumption_token),
            ResumptionTokenAlreadyUsedSnafu
        );
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_resumption_token(
        &mut self,
        resumption_token: &ResumptionToken,
    ) -> Result<bool, SignalingStorageError> {
        Ok(state().write().delete_resumption_token(resumption_token))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn try_acquire_participant_id(
        &mut self,
        participant_id: ParticipantId,
        runner_id: RunnerId,
    ) -> Result<bool, SignalingStorageError> {
        Ok(state()
            .write()
            .try_acquire_participant_id(participant_id, runner_id))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn participant_id_in_use(
        &mut self,
        participant: ParticipantId,
    ) -> Result<bool, SignalingStorageError> {
        Ok(state().read().participant_id_in_use(participant))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn release_participant_id(
        &mut self,
        participant: ParticipantId,
    ) -> Result<Option<RunnerId>, SignalingStorageError> {
        Ok(state().write().release_participant_id(participant))
    }
}

#[cfg(test)]
mod tests {
    use opentalk_signaling_core::VolatileStaticMemoryStorage;
    use serial_test::serial;

    use super::{super::super::test_common, state};

    fn storage() -> VolatileStaticMemoryStorage {
        state().write().reset();
        VolatileStaticMemoryStorage
    }

    #[tokio::test]
    #[serial]
    async fn ticket_token() {
        test_common::ticket_token(&mut storage()).await;
    }

    #[tokio::test]
    #[serial]
    async fn resumption_token() {
        test_common::resumption_token(&mut storage()).await;
    }

    #[tokio::test]
    #[serial]
    async fn participant_runner_lock() {
        test_common::participant_runner_lock(&mut storage()).await;
    }

    #[tokio::test]
    #[serial]
    async fn try_acquire_participant_id() {
        test_common::try_acquire_participant_id(&mut storage()).await;
    }
}
