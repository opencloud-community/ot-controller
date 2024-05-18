// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    sync::{Arc, OnceLock},
    time::Duration,
};

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, VolatileStaticMemoryStorage};
use opentalk_types::core::RoomId;
use parking_lot::RwLock;

use super::memory::MemoryBreakoutState;
use crate::api::signaling::breakout::storage::{BreakoutConfig, BreakoutStorage};

static STATE: OnceLock<Arc<RwLock<MemoryBreakoutState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemoryBreakoutState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl BreakoutStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_breakout_config(
        &mut self,
        room: RoomId,
        config: &BreakoutConfig,
    ) -> Result<Option<Duration>, SignalingModuleError> {
        Ok(state().write().set_config(room, config))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_breakout_config(
        &mut self,
        room: RoomId,
    ) -> Result<Option<BreakoutConfig>, SignalingModuleError> {
        Ok(state().read().get_config(room))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn del_breakout_config(&mut self, room: RoomId) -> Result<bool, SignalingModuleError> {
        Ok(state().write().del_config(room))
    }
}

#[cfg(test)]
mod tests {
    use opentalk_signaling_core::VolatileStaticMemoryStorage;
    use serial_test::serial;

    use super::{super::super::test_common, state};

    async fn storage() -> VolatileStaticMemoryStorage {
        state().write().reset();
        VolatileStaticMemoryStorage
    }

    #[tokio::test]
    #[serial]
    async fn config_unlimited() {
        test_common::config_unlimited(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn config_expiring() {
        test_common::config_expiring(&mut storage().await).await;
    }
}
