// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::SignalingRoomId;
use opentalk_types_signaling_automod::config::{Parameter, SelectionStrategy};
use rand::{Rng, seq::IndexedRandom};

use super::{Error, StateMachineOutput};
use crate::{
    exchange,
    storage::{AutomodStorage, StorageConfig},
};

/// Depending on the config will select a random participant to be speaker. This may be used when
/// the selection_strategy ist `random` or a moderator issues a `Select::Random` command.
pub async fn select_random<R: Rng>(
    storage: &mut dyn AutomodStorage,
    room: SignalingRoomId,
    config: &StorageConfig,
    rng: &mut R,
) -> Result<Option<StateMachineOutput>, Error> {
    let participant = match &config.parameter {
        Parameter {
            selection_strategy:
                SelectionStrategy::None | SelectionStrategy::Random | SelectionStrategy::Nomination,
            allow_double_selection,
            ..
        } => {
            if config.parameter.animation_on_random {
                let pool: Vec<opentalk_types_signaling::ParticipantId> = storage
                    .allow_list_get_all(room)
                    .await?
                    .into_iter()
                    .collect();

                // Special case: only one participant in pool -> skip animation
                if let [participant_id] = pool[..] {
                    // if double selection is disabled remove the last participant
                    // in theory this could also be just a DEL
                    if !allow_double_selection {
                        storage.allow_list_remove(room, participant_id).await?;
                    }

                    Some(participant_id)
                } else {
                    let selection = pool.choose(rng).copied();

                    if let Some(result) = selection {
                        return Ok(Some(StateMachineOutput::StartAnimation(
                            exchange::StartAnimation { pool, result },
                        )));
                    } else {
                        None
                    }
                }
            } else if *allow_double_selection {
                // GET RANDOM MEMBER FROM ALLOW_LIST
                storage.allow_list_random(room).await?
            } else {
                // POP RANDOM MEMBER FROM ALLOW_LIST
                storage.allow_list_pop_random(room).await?
            }
        }
        Parameter {
            selection_strategy: SelectionStrategy::Playlist,
            ..
        } => {
            // GET RANDOM MEMBER FROM PLAYLIST, REMOVE FROM PLAYLIST
            let playlist = storage.playlist_get_all(room).await?;

            if let Some(participant) = playlist.choose(rng).copied() {
                storage.playlist_remove_first(room, participant).await?;

                Some(participant)
            } else {
                None
            }
        }
    };

    super::map_select_unchecked(super::select_unchecked(storage, room, config, participant).await)
}

#[cfg(test)]
mod test {
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::{assert_eq, assert_ne};
    use serial_test::serial;

    use super::*;
    use crate::{
        state_machine::{
            StateMachineOutput,
            test::{ROOM, rng, setup_memory, setup_redis, unix_epoch},
        },
        storage::{Entry, EntryKind},
    };

    #[tokio::test]
    #[serial]
    async fn history_returns_since_redis() {
        history_returns_since(&mut setup_redis().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn history_returns_since_memory() {
        history_returns_since(&mut setup_memory().await).await;
    }

    /// Test that our storage works and always just returns the entries since the specified date
    /// 3 entries are added. Two before t and one after t. Only the one after t should be returned.
    async fn history_returns_since(storage: &mut dyn AutomodStorage) {
        let p1 = ParticipantId::from_u128(1);
        let p2 = ParticipantId::from_u128(2);
        let p3 = ParticipantId::from_u128(3);

        storage
            .history_add(
                ROOM,
                Entry {
                    timestamp: unix_epoch(1),
                    participant: p1,
                    kind: EntryKind::Start,
                },
            )
            .await
            .unwrap();

        storage
            .history_add(
                ROOM,
                Entry {
                    timestamp: unix_epoch(2),
                    participant: p1,
                    kind: EntryKind::Stop,
                },
            )
            .await
            .unwrap();

        storage
            .history_add(
                ROOM,
                Entry {
                    timestamp: unix_epoch(3),
                    participant: p3,
                    kind: EntryKind::Start,
                },
            )
            .await
            .unwrap();

        storage
            .history_add(
                ROOM,
                Entry {
                    timestamp: unix_epoch(4),
                    participant: p3,
                    kind: EntryKind::Stop,
                },
            )
            .await
            .unwrap();

        storage
            .history_add(
                ROOM,
                Entry {
                    timestamp: unix_epoch(10),
                    participant: p2,
                    kind: EntryKind::Start,
                },
            )
            .await
            .unwrap();

        let history = storage.history_get(ROOM, unix_epoch(5)).await.unwrap();

        assert_eq!(history, vec![p2]);
    }

    #[tokio::test]
    #[serial]
    async fn history_addition_redis() {
        history_addition(&mut setup_redis().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn history_addition_memory() {
        history_addition(&mut setup_memory().await).await;
    }

    /// Test that history works when selecting a random member
    /// 3 entries are added, assert that every time select_random returns an entry, it is also appended to the history.
    async fn history_addition(storage: &mut dyn AutomodStorage) {
        let mut rng = rng();

        let p1 = ParticipantId::from_u128(1);
        let p2 = ParticipantId::from_u128(2);
        let p3 = ParticipantId::from_u128(3);

        storage.allow_list_set(ROOM, &[p1, p2, p3]).await.unwrap();

        let config = StorageConfig {
            started: unix_epoch(0),
            issued_by: p1,
            parameter: Parameter {
                selection_strategy: SelectionStrategy::None,
                show_list: false,
                consider_hand_raise: false,
                time_limit: None,
                allow_double_selection: false,
                animation_on_random: false,
                auto_append_on_join: false,
            },
        };

        // === SELECT FIRST
        assert!(matches!(
            select_random(storage, ROOM, &config, &mut rng)
                .await
                .unwrap(),
            Some(StateMachineOutput::SpeakerUpdate(_))
        ));

        let first = storage.speaker_get(ROOM).await.unwrap().unwrap();

        assert_eq!(
            storage.history_get(ROOM, unix_epoch(0)).await.unwrap(),
            vec![first]
        );

        // === SELECT SECOND
        select_random(storage, ROOM, &config, &mut rng)
            .await
            .unwrap();

        let second = storage.speaker_get(ROOM).await.unwrap().unwrap();

        assert_ne!(first, second);

        assert_eq!(
            storage.history_get(ROOM, unix_epoch(0)).await.unwrap(),
            vec![first, second],
            "History is: {:#?}",
            storage
                .history_get_entries(ROOM, unix_epoch(0))
                .await
                .unwrap()
        );

        // === SELECT THIRD
        select_random(storage, ROOM, &config, &mut rng)
            .await
            .unwrap();

        let third = storage.speaker_get(ROOM).await.unwrap().unwrap();

        assert_ne!(first, third);
        assert_ne!(second, third);

        assert_eq!(
            storage.history_get(ROOM, unix_epoch(0)).await.unwrap(),
            vec![first, second, third]
        );
    }

    #[tokio::test]
    #[serial]
    async fn start_animation_redis() {
        start_animation(&mut setup_redis().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn start_animation_memory() {
        start_animation(&mut setup_memory().await).await;
    }

    /// Tests that random selection returns a StartAnimation response when animation_on_random is true
    async fn start_animation(storage: &mut dyn AutomodStorage) {
        let mut rng = rng();

        let p1 = ParticipantId::from_u128(1);
        let p2 = ParticipantId::from_u128(2);
        let p3 = ParticipantId::from_u128(3);

        storage.allow_list_set(ROOM, &[p1, p2, p3]).await.unwrap();

        let config = StorageConfig {
            started: unix_epoch(0),
            issued_by: p1,
            parameter: Parameter {
                selection_strategy: SelectionStrategy::None,
                show_list: false,
                consider_hand_raise: false,
                time_limit: None,
                allow_double_selection: false,
                animation_on_random: true,
                auto_append_on_join: false,
            },
        };

        assert!(matches!(
            select_random(storage, ROOM, &config, &mut rng)
                .await
                .unwrap(),
            Some(StateMachineOutput::StartAnimation(_))
        ));
    }

    #[tokio::test]
    #[serial]
    async fn select_random_when_none_redis() {
        select_random_when_none(&mut setup_redis().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn select_random_when_none_memory() {
        select_random_when_none(&mut setup_memory().await).await;
    }

    /// Test random selection when selection_strategy is None and double selection is forbidden
    /// 3 entries are added to the allow_list, two entries are added to the history.
    /// Assert that the third entry is returned by select_random
    async fn select_random_when_none(storage: &mut dyn AutomodStorage) {
        let mut rng = rng();

        let p1 = ParticipantId::from_u128(1);
        let p2 = ParticipantId::from_u128(2);
        let p3 = ParticipantId::from_u128(3);

        storage.allow_list_set(ROOM, &[p1, p2, p3]).await.unwrap();

        let config = StorageConfig {
            started: unix_epoch(0),
            issued_by: p1,
            parameter: Parameter {
                selection_strategy: SelectionStrategy::None,
                show_list: false,
                consider_hand_raise: false,
                time_limit: None,
                allow_double_selection: false,
                animation_on_random: false,
                auto_append_on_join: false,
            },
        };

        select_random(storage, ROOM, &config, &mut rng)
            .await
            .unwrap();

        let speaker = storage.speaker_get(ROOM).await.unwrap().unwrap();

        assert!([p1, p2, p3].contains(&speaker));
    }

    #[tokio::test]
    #[serial]
    async fn select_random_when_playlist_redis() {
        select_random_when_playlist(&mut setup_redis().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn select_random_when_playlist_memory() {
        select_random_when_playlist(&mut setup_memory().await).await;
    }

    /// Test random selection when selection_strategy is Playlist
    /// 3 entries are added to the playlist, one entry is added to the history (stopped).
    /// Assert that select_random removes the entries from playlist and adds them to the history.
    async fn select_random_when_playlist(storage: &mut dyn AutomodStorage) {
        let mut rng = rng();

        let p1 = ParticipantId::from_u128(1);
        let p2 = ParticipantId::from_u128(2);
        let p3 = ParticipantId::from_u128(3);

        storage.playlist_set(ROOM, &[p1, p2, p3]).await.unwrap();

        storage
            .history_add(
                ROOM,
                Entry {
                    timestamp: unix_epoch(1),
                    participant: p1,
                    kind: EntryKind::Start,
                },
            )
            .await
            .unwrap();

        storage
            .history_add(
                ROOM,
                Entry {
                    timestamp: unix_epoch(2),
                    participant: p1,
                    kind: EntryKind::Stop,
                },
            )
            .await
            .unwrap();

        let config = StorageConfig {
            started: unix_epoch(0),
            issued_by: p1,
            parameter: Parameter {
                selection_strategy: SelectionStrategy::Playlist,
                show_list: false,
                consider_hand_raise: false,
                time_limit: None,
                allow_double_selection: false,
                animation_on_random: false,
                auto_append_on_join: false,
            },
        };

        // === SELECT FIRST
        select_random(storage, ROOM, &config, &mut rng)
            .await
            .unwrap();

        let speaker = storage.speaker_get(ROOM).await.unwrap().unwrap();

        assert_eq!(speaker, p3);
        assert_eq!(storage.playlist_get_all(ROOM).await.unwrap(), vec![p1, p2]);

        // === SELECT SECOND
        select_random(storage, ROOM, &config, &mut rng)
            .await
            .unwrap();

        let speaker = storage.speaker_get(ROOM).await.unwrap().unwrap();

        assert_eq!(speaker, p2);
        assert_eq!(storage.playlist_get_all(ROOM).await.unwrap(), vec![p1]);

        // === SELECT THIRD
        select_random(storage, ROOM, &config, &mut rng)
            .await
            .unwrap();

        let speaker = storage.speaker_get(ROOM).await.unwrap().unwrap();

        assert_eq!(speaker, p1);
        assert_eq!(storage.playlist_get_all(ROOM).await.unwrap(), vec![]);

        // === SELECT LAST MUST BE NONE
        select_random(storage, ROOM, &config, &mut rng)
            .await
            .unwrap();

        assert_eq!(storage.speaker_get(ROOM).await.unwrap(), None);
    }

    #[tokio::test]
    #[serial]
    async fn select_random_when_random_allow_double_select_redis() {
        select_random_when_random_allow_double_select(&mut setup_redis().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn select_random_when_random_allow_double_select_memory() {
        select_random_when_random_allow_double_select(&mut setup_memory().await).await;
    }

    /// Test random selection when selection_strategy is Random and reselection is allowed
    /// 3 entries are added to the allow_list. Select 4 times. Assert that at least once a double selection was encountered
    async fn select_random_when_random_allow_double_select(storage: &mut dyn AutomodStorage) {
        let mut rng = rng();

        let p1 = ParticipantId::from_u128(1);
        let p2 = ParticipantId::from_u128(2);
        let p3 = ParticipantId::from_u128(3);

        storage.allow_list_set(ROOM, &[p1, p2, p3]).await.unwrap();

        storage
            .history_add(
                ROOM,
                Entry {
                    timestamp: unix_epoch(1),
                    participant: p1,
                    kind: EntryKind::Start,
                },
            )
            .await
            .unwrap();

        storage
            .history_add(
                ROOM,
                Entry {
                    timestamp: unix_epoch(2),
                    participant: p1,
                    kind: EntryKind::Stop,
                },
            )
            .await
            .unwrap();

        let config = StorageConfig {
            started: unix_epoch(0),
            issued_by: p1,
            parameter: Parameter {
                selection_strategy: SelectionStrategy::Random,
                show_list: false,
                consider_hand_raise: false,
                time_limit: None,
                allow_double_selection: true,
                animation_on_random: false,
                auto_append_on_join: false,
            },
        };

        // === SELECT FIRST
        let mut selected = Vec::new();

        for _ in 0..4 {
            select_random(storage, ROOM, &config, &mut rng)
                .await
                .unwrap();

            let speaker = storage.speaker_get(ROOM).await.unwrap().unwrap();

            if selected.contains(&speaker) {
                return;
            } else {
                selected.push(speaker);
            }
        }

        panic!("selected did not contain any duplicates ???")
    }
}
