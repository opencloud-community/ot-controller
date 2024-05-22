// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod redis;
mod shared_folder_storage;
mod volatile;

pub(crate) use redis::{
    delete_shared_folder, delete_shared_folder_initialized, get_shared_folder,
    is_shared_folder_initialized, set_shared_folder,
};
pub(crate) use shared_folder_storage::SharedFolderStorage;

#[cfg(test)]
mod test_common {

    use opentalk_signaling_core::SignalingRoomId;

    use super::SharedFolderStorage;

    const ROOM: SignalingRoomId = SignalingRoomId::nil();

    pub(super) async fn initialized(storage: &mut dyn SharedFolderStorage) {
        storage.set_shared_folder_initialized(ROOM).await.unwrap();
    }
}
