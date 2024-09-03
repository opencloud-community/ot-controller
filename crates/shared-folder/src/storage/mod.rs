// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod redis;
mod shared_folder_storage;
mod volatile;

pub(crate) use shared_folder_storage::SharedFolderStorage;

#[cfg(test)]
mod test_common {

    use opentalk_signaling_core::SignalingRoomId;
    use opentalk_types::common::shared_folder::SharedFolder;
    use opentalk_types_common::shared_folders::SharedFolderAccess;

    use super::SharedFolderStorage;

    const ROOM: SignalingRoomId = SignalingRoomId::nil();

    pub(super) async fn initialized(storage: &mut dyn SharedFolderStorage) {
        assert!(!storage.is_shared_folder_initialized(ROOM).await.unwrap());

        storage.set_shared_folder_initialized(ROOM).await.unwrap();

        assert!(storage.is_shared_folder_initialized(ROOM).await.unwrap());

        storage
            .delete_shared_folder_initialized(ROOM)
            .await
            .unwrap();

        assert!(!storage.is_shared_folder_initialized(ROOM).await.unwrap());
    }

    pub(super) async fn shared_folder(storage: &mut dyn SharedFolderStorage) {
        let shared_folder = SharedFolder {
            read: SharedFolderAccess {
                url: "http://example.com".to_owned(),
                password: "password123".to_owned(),
            },
            read_write: None,
        };

        assert!(storage.get_shared_folder(ROOM).await.unwrap().is_none());

        storage
            .set_shared_folder(ROOM, shared_folder.clone())
            .await
            .unwrap();

        assert_eq!(
            Some(shared_folder),
            storage.get_shared_folder(ROOM).await.unwrap()
        );

        storage.delete_shared_folder(ROOM).await.unwrap();

        assert!(storage.get_shared_folder(ROOM).await.unwrap().is_none());
    }
}
