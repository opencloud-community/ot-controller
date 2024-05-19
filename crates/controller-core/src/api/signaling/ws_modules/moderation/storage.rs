// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod moderation_storage;
mod redis;
mod volatile;

pub(crate) use moderation_storage::ModerationStorage;
// TODO: remove once everything is exposed through the ModerationStorage trait.
pub(crate) use redis::{
    delete_raise_hands_enabled, delete_waiting_room, delete_waiting_room_accepted,
    is_raise_hands_enabled, set_raise_hands_enabled, waiting_room_accepted_add,
    waiting_room_accepted_all, waiting_room_accepted_len, waiting_room_accepted_remove,
    waiting_room_accepted_remove_list, waiting_room_add, waiting_room_all, waiting_room_contains,
    waiting_room_len, waiting_room_remove,
};

#[cfg(test)]
mod test_common {
    use opentalk_types::core::{RoomId, UserId};
    use pretty_assertions::assert_eq;

    use super::ModerationStorage;

    pub const ROOM: RoomId = RoomId::nil();
    pub const BOB: UserId = UserId::from_u128(0xdeadbeef);
    pub const ALICE: UserId = UserId::from_u128(0xbadcafe);

    pub(super) async fn user_bans(storage: &mut dyn ModerationStorage) {
        assert!(!storage.is_user_banned(ROOM, BOB).await.unwrap());
        assert!(!storage.is_user_banned(ROOM, ALICE).await.unwrap());

        storage.ban_user(ROOM, BOB).await.unwrap();

        assert!(storage.is_user_banned(ROOM, BOB).await.unwrap());
        assert!(!storage.is_user_banned(ROOM, ALICE).await.unwrap());

        storage.ban_user(ROOM, ALICE).await.unwrap();

        assert!(storage.is_user_banned(ROOM, BOB).await.unwrap());
        assert!(storage.is_user_banned(ROOM, ALICE).await.unwrap());

        storage.delete_user_bans(ROOM).await.unwrap();

        assert!(!storage.is_user_banned(ROOM, BOB).await.unwrap());
        assert!(!storage.is_user_banned(ROOM, ALICE).await.unwrap());
    }

    pub(super) async fn waiting_room_enabled_flag(storage: &mut dyn ModerationStorage) {
        assert!(!storage.is_waiting_room_enabled(ROOM).await.unwrap());

        assert_eq!(
            storage.init_waiting_room_enabled(ROOM, true).await.unwrap(),
            true
        );

        assert!(storage.is_waiting_room_enabled(ROOM).await.unwrap());

        // Ensure that the waiting room flag will not be changed when attempting to initialize
        // after it has been initialized already
        assert_eq!(
            storage
                .init_waiting_room_enabled(ROOM, false)
                .await
                .unwrap(),
            true
        );
        assert!(storage.is_waiting_room_enabled(ROOM).await.unwrap());

        storage.set_waiting_room_enabled(ROOM, false).await.unwrap();

        assert!(!storage.is_waiting_room_enabled(ROOM).await.unwrap());

        storage.set_waiting_room_enabled(ROOM, true).await.unwrap();
        storage.delete_waiting_room_enabled(ROOM).await.unwrap();

        assert!(!storage.is_waiting_room_enabled(ROOM).await.unwrap());
    }
}
