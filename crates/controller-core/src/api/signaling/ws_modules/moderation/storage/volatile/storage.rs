// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, VolatileStaticMemoryStorage};
use opentalk_types::core::{RoomId, UserId};
use parking_lot::RwLock;

use super::memory::MemoryModerationState;
use crate::api::signaling::moderation::storage::ModerationStorage;

static STATE: OnceLock<Arc<RwLock<MemoryModerationState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemoryModerationState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl ModerationStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn ban_user(&mut self, room: RoomId, user: UserId) -> Result<(), SignalingModuleError> {
        state().write().ban_user(room, user);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn is_user_banned(
        &mut self,
        room: RoomId,
        user: UserId,
    ) -> Result<bool, SignalingModuleError> {
        Ok(state().read().is_user_banned(room, user))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_user_bans(&mut self, room: RoomId) -> Result<(), SignalingModuleError> {
        state().write().delete_user_bans(room);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use opentalk_signaling_core::VolatileStaticMemoryStorage;
    use serial_test::serial;

    use super::{super::super::test_common, state};

    async fn storage() -> VolatileStaticMemoryStorage {
        state().write().reset();
        VolatileStaticMemoryStorage
    }

    #[tokio::test]
    #[serial]
    async fn user_bans() {
        test_common::user_bans(&mut storage().await).await;
    }
}
