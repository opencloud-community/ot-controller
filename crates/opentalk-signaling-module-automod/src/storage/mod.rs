// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains modules of operations on redis used by the automod

mod automod_storage;
mod redis;
mod storage_config;
mod volatile;

pub(crate) use automod_storage::{AutomodStorage, Entry, EntryKind};
pub use storage_config::StorageConfig;
#[cfg(test)]
pub(crate) use volatile::storage::tests::reset_state as reset_memory_state;

#[cfg(test)]
pub(crate) mod test_common {
    use std::collections::BTreeSet;

    use chrono::DateTime;
    use opentalk_signaling_core::SignalingRoomId;
    use opentalk_types_signaling::ParticipantId;
    use opentalk_types_signaling_automod::config::{Parameter, SelectionStrategy};
    use pretty_assertions::assert_eq;

    use super::AutomodStorage;
    use crate::storage::{Entry, EntryKind, StorageConfig};

    pub(crate) const ROOM: SignalingRoomId = SignalingRoomId::nil();
    const ALICE: ParticipantId = ParticipantId::from_u128(0xa11c3);
    const BOB: ParticipantId = ParticipantId::from_u128(0x808);
    const CHARLIE: ParticipantId = ParticipantId::from_u128(0xca11e);

    pub(crate) async fn playlist(storage: &mut dyn AutomodStorage) {
        assert!(storage.playlist_get_all(ROOM).await.unwrap().is_empty());
        storage.playlist_push(ROOM, ALICE).await.unwrap();

        assert_eq!(vec![ALICE], storage.playlist_get_all(ROOM).await.unwrap());

        storage.playlist_push(ROOM, ALICE).await.unwrap();
        storage.playlist_push(ROOM, BOB).await.unwrap();
        storage.playlist_push(ROOM, ALICE).await.unwrap();
        assert_eq!(
            vec![ALICE, ALICE, BOB, ALICE],
            storage.playlist_get_all(ROOM).await.unwrap()
        );

        assert_eq!(Some(ALICE), storage.playlist_pop(ROOM).await.unwrap());
        assert_eq!(
            vec![ALICE, BOB, ALICE],
            storage.playlist_get_all(ROOM).await.unwrap()
        );

        assert_eq!(
            2,
            storage
                .playlist_remove_all_occurrences(ROOM, ALICE)
                .await
                .unwrap()
        );
        assert_eq!(
            0,
            storage
                .playlist_remove_all_occurrences(ROOM, ALICE)
                .await
                .unwrap()
        );
        assert_eq!(vec![BOB], storage.playlist_get_all(ROOM).await.unwrap());

        let playlist = &[ALICE, BOB, ALICE, BOB, BOB, ALICE, ALICE];
        storage.playlist_set(ROOM, playlist).await.unwrap();
        assert_eq!(playlist, &storage.playlist_get_all(ROOM).await.unwrap()[..]);
    }

    pub(crate) async fn playlist_remove_first(storage: &mut dyn AutomodStorage) {
        let playlist = &[ALICE, BOB, ALICE, BOB, BOB, ALICE, ALICE];
        storage.playlist_set(ROOM, playlist).await.unwrap();

        storage.playlist_remove_first(ROOM, BOB).await.unwrap();

        assert_eq!(
            &[ALICE, ALICE, BOB, BOB, ALICE, ALICE],
            &storage.playlist_get_all(ROOM).await.unwrap()[..]
        );

        storage.playlist_remove_first(ROOM, CHARLIE).await.unwrap();

        assert_eq!(
            &[ALICE, ALICE, BOB, BOB, ALICE, ALICE],
            &storage.playlist_get_all(ROOM).await.unwrap()[..]
        );
    }

    pub(crate) async fn allow_list(storage: &mut dyn AutomodStorage) {
        storage
            .allow_list_set(ROOM, &[ALICE, BOB, CHARLIE, BOB, ALICE])
            .await
            .unwrap();
        assert_eq!(
            BTreeSet::from([ALICE, BOB, CHARLIE]),
            storage.allow_list_get_all(ROOM).await.unwrap()
        );
        storage.allow_list_remove(ROOM, ALICE).await.unwrap();
        assert_eq!(
            BTreeSet::from([BOB, CHARLIE]),
            storage.allow_list_get_all(ROOM).await.unwrap()
        );

        let random_participant = storage
            .allow_list_random(ROOM)
            .await
            .unwrap()
            .expect("The allow list must contain entries");
        assert!(
            storage
                .allow_list_contains(ROOM, random_participant)
                .await
                .unwrap()
        );

        storage.allow_list_add(ROOM, ALICE).await.unwrap();
        assert_eq!(
            BTreeSet::from([BOB, CHARLIE, ALICE]),
            storage.allow_list_get_all(ROOM).await.unwrap()
        );

        let popped = storage
            .allow_list_pop_random(ROOM)
            .await
            .unwrap()
            .expect("The allow list must contain entries");
        assert!(!storage.allow_list_contains(ROOM, popped).await.unwrap());

        storage.allow_list_delete(ROOM).await.unwrap();
        assert!(storage.allow_list_get_all(ROOM).await.unwrap().is_empty());
    }

    pub(crate) async fn storage_config(storage: &mut dyn AutomodStorage) {
        assert!(storage.config_get(ROOM).await.unwrap().is_none());
        assert!(!storage.config_exists(ROOM).await.unwrap());

        let config = StorageConfig {
            started: Default::default(),
            issued_by: ALICE,
            parameter: Parameter {
                selection_strategy: SelectionStrategy::Nomination,
                show_list: Default::default(),
                consider_hand_raise: Default::default(),
                time_limit: Default::default(),
                allow_double_selection: Default::default(),
                animation_on_random: Default::default(),
                auto_append_on_join: Default::default(),
            },
        };
        storage.config_set(ROOM, config.clone()).await.unwrap();

        assert_eq!(Some(config), storage.config_get(ROOM).await.unwrap());
        assert!(storage.config_exists(ROOM).await.unwrap());

        storage.config_delete(ROOM).await.unwrap();
        assert!(!storage.config_exists(ROOM).await.unwrap());
    }

    pub(crate) async fn speaker(storage: &mut dyn AutomodStorage) {
        assert!(storage.speaker_get(ROOM).await.unwrap().is_none());
        storage.speaker_set(ROOM, ALICE).await.unwrap();
        assert_eq!(Some(ALICE), storage.speaker_get(ROOM).await.unwrap());

        storage.speaker_set(ROOM, BOB).await.unwrap();
        assert_eq!(Some(BOB), storage.speaker_get(ROOM).await.unwrap());

        storage.speaker_delete(ROOM).await.unwrap();
        assert_eq!(None, storage.speaker_get(ROOM).await.unwrap());
    }

    pub(crate) async fn history(storage: &mut dyn AutomodStorage) {
        let date0 = DateTime::from_timestamp(0, 0).unwrap();
        let date1 = DateTime::from_timestamp(1, 0).unwrap();
        let date2 = DateTime::from_timestamp(2, 0).unwrap();
        // we compare with an empty vec instead of using Vec::is_empty so that we can see the content of the Vec in the error output.
        let empty = Vec::<ParticipantId>::new();

        assert_eq!(empty, storage.history_get(ROOM, date0).await.unwrap());
        let entry_at_alice_1 = Entry {
            timestamp: date1,
            participant: ALICE,
            kind: EntryKind::Start,
        };
        let entry_bob_at_0 = Entry {
            timestamp: date0,
            participant: BOB,
            kind: EntryKind::Start,
        };
        let entry_bob_at_1 = Entry {
            timestamp: date1,
            participant: BOB,
            kind: EntryKind::Start,
        };

        storage.history_add(ROOM, entry_at_alice_1).await.unwrap();
        storage.history_add(ROOM, entry_at_alice_1).await.unwrap();
        storage.history_add(ROOM, entry_bob_at_1).await.unwrap();
        storage.history_add(ROOM, entry_bob_at_0).await.unwrap();

        assert_eq!(
            vec![BOB, BOB, ALICE],
            storage.history_get(ROOM, date0).await.unwrap()
        );
        assert_eq!(
            vec![BOB, ALICE],
            storage.history_get(ROOM, date1).await.unwrap()
        );
        assert_eq!(empty, storage.history_get(ROOM, date2).await.unwrap());
    }

    /// Alice speaks twice in a row
    pub(crate) async fn history_repeated_speaker(storage: &mut dyn AutomodStorage) {
        let date0 = DateTime::from_timestamp(0, 0).unwrap();
        let date1 = DateTime::from_timestamp(1, 0).unwrap();
        let date2 = DateTime::from_timestamp(2, 0).unwrap();
        // we compare with an empty vec instead of using Vec::is_empty so that we can see the content of the Vec in the error output.
        let empty = Vec::<ParticipantId>::new();

        assert_eq!(empty, storage.history_get(ROOM, date0).await.unwrap());

        storage
            .history_add(
                ROOM,
                Entry {
                    timestamp: date0,
                    participant: ALICE,
                    kind: EntryKind::Start,
                },
            )
            .await
            .unwrap();
        storage
            .history_add(
                ROOM,
                Entry {
                    timestamp: date1,
                    participant: ALICE,
                    kind: EntryKind::Stop,
                },
            )
            .await
            .unwrap();
        storage
            .history_add(
                ROOM,
                Entry {
                    timestamp: date2,
                    participant: ALICE,
                    kind: EntryKind::Start,
                },
            )
            .await
            .unwrap();

        assert_eq!(
            vec![ALICE, ALICE],
            storage.history_get(ROOM, date0).await.unwrap()
        );
    }
}
