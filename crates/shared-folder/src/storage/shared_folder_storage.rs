// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId};

#[async_trait(?Send)]
pub(crate) trait SharedFolderStorage {
    async fn set_shared_folder_initialized(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError>;
}
