// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use opentalk_signaling_core::VolatileStaticMemoryStorage;
use opentalk_types::core::TicketToken;
use parking_lot::RwLock;

use super::memory::MemorySignalingState;
use crate::api::signaling::{
    storage::{SignalingStorage, SignalingStorageError},
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
}

#[cfg(test)]
mod test {
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
}
