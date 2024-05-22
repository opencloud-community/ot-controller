// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId};

#[async_trait(?Send)]
pub(crate) trait RecordingStorage {
    async fn is_streaming_initialized(
        &mut self,
        room_id: SignalingRoomId,
    ) -> Result<bool, SignalingModuleError>;
}
