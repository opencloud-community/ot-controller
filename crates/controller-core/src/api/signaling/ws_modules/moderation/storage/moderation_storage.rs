// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::SignalingModuleError;
use opentalk_types::core::{RoomId, UserId};

#[async_trait(?Send)]
pub(crate) trait ModerationStorage {
    async fn ban_user(&mut self, room: RoomId, user: UserId) -> Result<(), SignalingModuleError>;
}
