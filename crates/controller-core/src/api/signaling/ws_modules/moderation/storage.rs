// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod moderation_storage;
mod redis;
mod volatile;

pub(crate) use moderation_storage::ModerationStorage;
// TODO: remove once everything is exposed through the ModerationStorage trait.
pub(crate) use redis::{
    delete_waiting_room_accepted, waiting_room_accepted_add, waiting_room_accepted_all,
    waiting_room_accepted_len, waiting_room_accepted_remove, waiting_room_accepted_remove_list,
};

#[cfg(test)]
mod test_common {
    use std::collections::BTreeSet;

    use opentalk_types::core::{ParticipantId, RoomId, UserId};
    use pretty_assertions::assert_eq;

    use super::ModerationStorage;

    pub const ROOM: RoomId = RoomId::nil();
    pub const BOB_USER: UserId = UserId::from_u128(0xdeadbeef);
    pub const ALICE_USER: UserId = UserId::from_u128(0xbadcafe);
    pub const BOB_PARTICIPANT: ParticipantId = ParticipantId::from_u128(0xdeadbeef);
    pub const ALICE_PARTICIPANT: ParticipantId = ParticipantId::from_u128(0xbadcafe);

    pub(super) async fn user_bans(storage: &mut dyn ModerationStorage) {
        assert!(!storage.is_user_banned(ROOM, BOB_USER).await.unwrap());
        assert!(!storage.is_user_banned(ROOM, ALICE_USER).await.unwrap());

        storage.ban_user(ROOM, BOB_USER).await.unwrap();

        assert!(storage.is_user_banned(ROOM, BOB_USER).await.unwrap());
        assert!(!storage.is_user_banned(ROOM, ALICE_USER).await.unwrap());

        storage.ban_user(ROOM, ALICE_USER).await.unwrap();

        assert!(storage.is_user_banned(ROOM, BOB_USER).await.unwrap());
        assert!(storage.is_user_banned(ROOM, ALICE_USER).await.unwrap());

        storage.delete_user_bans(ROOM).await.unwrap();

        assert!(!storage.is_user_banned(ROOM, BOB_USER).await.unwrap());
        assert!(!storage.is_user_banned(ROOM, ALICE_USER).await.unwrap());
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

    pub(super) async fn raise_hands_enabled_flag(storage: &mut dyn ModerationStorage) {
        assert!(storage.is_raise_hands_enabled(ROOM).await.unwrap());

        storage.set_raise_hands_enabled(ROOM, false).await.unwrap();

        assert!(!storage.is_raise_hands_enabled(ROOM).await.unwrap());

        storage.set_raise_hands_enabled(ROOM, true).await.unwrap();

        assert!(storage.is_raise_hands_enabled(ROOM).await.unwrap());

        storage.delete_raise_hands_enabled(ROOM).await.unwrap();

        assert!(storage.is_raise_hands_enabled(ROOM).await.unwrap());
    }

    pub(super) async fn waiting_room_participants(storage: &mut dyn ModerationStorage) {
        assert_eq!(
            storage.waiting_room_participant_count(ROOM).await.unwrap(),
            0usize
        );
        assert!(storage
            .waiting_room_add_participant(ROOM, BOB_PARTICIPANT)
            .await
            .unwrap());
        // Ensure that we receive `false` when attempting to add the same participant twice
        assert!(!storage
            .waiting_room_add_participant(ROOM, BOB_PARTICIPANT)
            .await
            .unwrap());
        storage
            .waiting_room_remove_participant(ROOM, BOB_PARTICIPANT)
            .await
            .unwrap();
        // Now the same participant can be added again
        assert!(storage
            .waiting_room_add_participant(ROOM, BOB_PARTICIPANT)
            .await
            .unwrap());

        assert_eq!(
            storage.waiting_room_participants(ROOM).await.unwrap(),
            BTreeSet::from_iter([BOB_PARTICIPANT])
        );
        assert_eq!(
            storage.waiting_room_participant_count(ROOM).await.unwrap(),
            1usize
        );

        assert!(storage
            .waiting_room_add_participant(ROOM, ALICE_PARTICIPANT)
            .await
            .unwrap());

        assert_eq!(
            storage.waiting_room_participants(ROOM).await.unwrap(),
            BTreeSet::from_iter([BOB_PARTICIPANT, ALICE_PARTICIPANT])
        );
        assert_eq!(
            storage.waiting_room_participant_count(ROOM).await.unwrap(),
            2usize
        );

        storage.delete_waiting_room(ROOM).await.unwrap();

        assert_eq!(
            storage.waiting_room_participants(ROOM).await.unwrap(),
            BTreeSet::new()
        );
        assert_eq!(
            storage.waiting_room_participant_count(ROOM).await.unwrap(),
            0usize
        );
    }
}
