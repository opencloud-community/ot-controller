// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{
    control::storage::ControlEventStorage, SignalingModuleError, SignalingRoomId,
};
use opentalk_types::common::shared_folder::SharedFolder;

#[async_trait(?Send)]
pub(crate) trait SharedFolderStorage: ControlEventStorage {
    async fn set_shared_folder_initialized(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError>;

    async fn is_shared_folder_initialized(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<bool, SignalingModuleError>;

    async fn delete_shared_folder_initialized(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError>;

    async fn get_shared_folder(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<SharedFolder>, SignalingModuleError>;

    async fn set_shared_folder(
        &mut self,
        room: SignalingRoomId,
        value: SharedFolder,
    ) -> Result<(), SignalingModuleError>;

    async fn delete_shared_folder(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError>;
}
