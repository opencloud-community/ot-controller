// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::SignalingRoomId;
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_automod::config::{Parameter, SelectionStrategy};
use rand::Rng;

use super::{Error, StateMachineOutput};
use crate::storage::{AutomodStorage, StorageConfig};

/// Depending on the config will inspect/change the state_machine's state to select the next
/// user to be speaker.
pub async fn select_next<R: Rng>(
    storage: &mut dyn AutomodStorage,
    room: SignalingRoomId,
    config: &StorageConfig,
    user_selected: Option<ParticipantId>,
    rng: &mut R,
) -> Result<Option<StateMachineOutput>, Error> {
    let participant = match config.parameter {
        Parameter {
            selection_strategy: SelectionStrategy::None,
            ..
        } => None,
        Parameter {
            selection_strategy: SelectionStrategy::Playlist,
            ..
        } => storage.playlist_pop(room).await?,
        Parameter {
            selection_strategy: SelectionStrategy::Nomination,
            allow_double_selection,
            ..
        } => select_next_nomination(storage, room, user_selected, allow_double_selection).await?,
        Parameter {
            selection_strategy: SelectionStrategy::Random,
            ..
        } => return super::select_random(storage, room, config, rng).await,
    };

    super::map_select_unchecked(super::select_unchecked(storage, room, config, participant).await)
}

/// Returns the next (if any) participant to be selected inside a `Nomination` selection strategy.
async fn select_next_nomination(
    storage: &mut dyn AutomodStorage,
    room: SignalingRoomId,
    user_selected: Option<ParticipantId>,
    allow_double_selection: bool,
) -> Result<Option<ParticipantId>, Error> {
    // get user selection
    let participant = if let Some(participant) = user_selected {
        participant
    } else {
        // No next user nominated, unset current speaker
        return Ok(None);
    };

    // Different approaches depending on `allow_double_selection`
    if allow_double_selection {
        // Double selection is allowed:
        // Just check if the given participant is inside the allow_list
        if storage.allow_list_contains(room, participant).await? {
            Ok(Some(participant))
        } else {
            Err(Error::InvalidSelection)
        }
    } else {
        // Double selection is disallowed:
        // Try to remove the participant from the allow_list.
        // If the removed count is not 1 (which would indicate 1 item removed), the participant
        // wasn't inside the allow_list and thus an invalid selection was made
        let removed_count = storage.allow_list_remove(room, participant).await?;

        if removed_count == 1 {
            Ok(Some(participant))
        } else {
            Err(Error::InvalidSelection)
        }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use serial_test::serial;

    use super::*;
    use crate::{
        state_machine::{
            self, exchange,
            test::{rng, setup_memory, setup_redis, unix_epoch, ROOM},
        },
        storage::Entry,
    };

    fn assert_history_without_timestamp(lhs: &[Entry], rhs: &[Entry]) {
        assert_eq!(lhs.len(), rhs.len());
        lhs.iter().zip(rhs.iter()).for_each(|(lhs, rhs)| {
            assert_eq!(lhs.kind, rhs.kind);
            assert_eq!(lhs.participant, rhs.participant);
        })
    }

    #[tokio::test]
    #[serial]
    async fn start_animation_on_yield_redis() {
        start_animation_on_yield(&mut setup_redis().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn start_animation_on_yield_memory() {
        start_animation_on_yield(&mut setup_memory().await).await;
    }

    /// Next returns StartAnimation when animation_on_random is true
    async fn start_animation_on_yield(storage: &mut dyn AutomodStorage) {
        let mut rng = rng();

        let p1 = ParticipantId::from_u128(1);
        let p2 = ParticipantId::from_u128(2);
        let p3 = ParticipantId::from_u128(3);

        storage.allow_list_set(ROOM, &[p1, p2, p3]).await.unwrap();

        let config = StorageConfig {
            started: unix_epoch(0),
            issued_by: p1,
            parameter: Parameter {
                selection_strategy: SelectionStrategy::Random,
                show_list: false,
                consider_hand_raise: false,
                time_limit: None,
                allow_double_selection: false,
                animation_on_random: true,
                auto_append_on_join: false,
            },
        };

        assert!(matches!(
            select_next(storage, ROOM, &config, None, &mut rng)
                .await
                .unwrap(),
            Some(StateMachineOutput::StartAnimation(_))
        ));
    }

    #[tokio::test]
    #[serial]
    async fn nomination_reselection_allowed_redis() {
        nomination_reselection_allowed(&mut setup_redis().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn nomination_reselection_allowed_memory() {
        nomination_reselection_allowed(&mut setup_memory().await).await;
    }

    /// Test next when selection_strategy is Nomination and reselection is allowed
    async fn nomination_reselection_allowed(storage: &mut dyn AutomodStorage) {
        let mut rng = rng();

        let p1 = ParticipantId::from_u128(1);

        // Add current speaker
        storage.history_add(ROOM, Entry::start(p1)).await.unwrap();
        storage.allow_list_set(ROOM, &[p1]).await.unwrap();
        storage.speaker_set(ROOM, p1).await.unwrap();

        let config = StorageConfig {
            started: unix_epoch(0),
            issued_by: p1,
            parameter: Parameter {
                selection_strategy: SelectionStrategy::Nomination,
                show_list: false,
                consider_hand_raise: false,
                time_limit: None,
                allow_double_selection: true,
                animation_on_random: false,
                auto_append_on_join: false,
            },
        };

        // Check with nominee in history
        let next = select_next(storage, ROOM, &config, Some(p1), &mut rng)
            .await
            .unwrap();
        let history = storage.history_get(ROOM, unix_epoch(0)).await.unwrap();
        assert_eq!(history, vec![p1, p1]);

        let full_history = storage
            .history_get_entries(ROOM, unix_epoch(0))
            .await
            .unwrap();
        assert_history_without_timestamp(
            &full_history,
            &[Entry::start(p1), Entry::stop(p1), Entry::start(p1)],
        );

        let speaker = storage.speaker_get(ROOM).await.unwrap();
        assert_eq!(speaker, Some(p1));

        assert_eq!(
            next,
            Some(StateMachineOutput::SpeakerUpdate(exchange::SpeakerUpdate {
                speaker: Some(p1),
                history: Some(vec![p1, p1]),
                remaining: Some(vec![p1])
            }))
        );

        let config = StorageConfig {
            started: unix_epoch(0),
            issued_by: p1,
            parameter: Parameter {
                selection_strategy: SelectionStrategy::Nomination,
                show_list: false,
                consider_hand_raise: false,
                time_limit: None,
                allow_double_selection: false,
                animation_on_random: false,
                auto_append_on_join: false,
            },
        };

        // Check with nominee in history
        let next = select_next(storage, ROOM, &config, Some(p1), &mut rng)
            .await
            .unwrap();
        assert_eq!(
            next,
            Some(StateMachineOutput::SpeakerUpdate(exchange::SpeakerUpdate {
                speaker: Some(p1),
                history: Some(vec![p1, p1, p1]),
                remaining: Some(vec![])
            }))
        );

        let history = storage.history_get(ROOM, unix_epoch(0)).await.unwrap();
        assert_eq!(history, vec![p1, p1, p1]);

        let full_history = storage
            .history_get_entries(ROOM, unix_epoch(0))
            .await
            .unwrap();
        assert_history_without_timestamp(
            &full_history,
            &[
                Entry::start(p1),
                Entry::stop(p1),
                Entry::start(p1),
                Entry::stop(p1),
                Entry::start(p1),
            ],
        );

        let speaker = storage.speaker_get(ROOM).await.unwrap();
        assert_eq!(speaker, Some(p1));
    }

    #[tokio::test]
    #[serial]
    async fn nomination_redis() {
        nomination(&mut setup_redis().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn nomination_memory() {
        nomination(&mut setup_memory().await).await;
    }

    /// Test next when selection_strategy is Nomination and an allow_list containing only 2 of 3
    /// possible participants
    async fn nomination(storage: &mut dyn AutomodStorage) {
        let mut rng = rng();

        let p1 = ParticipantId::from_u128(1);
        let p2 = ParticipantId::from_u128(2);
        let p3 = ParticipantId::from_u128(3);

        storage.allow_list_set(ROOM, &[p1, p2]).await.unwrap();

        let config = StorageConfig {
            started: unix_epoch(0),
            issued_by: p1,
            parameter: Parameter {
                selection_strategy: SelectionStrategy::Nomination,
                show_list: false,
                consider_hand_raise: false,
                time_limit: None,
                allow_double_selection: false,
                animation_on_random: false,
                auto_append_on_join: false,
            },
        };
        // Check allowed participant
        let next = select_next(storage, ROOM, &config, Some(p1), &mut rng)
            .await
            .unwrap();
        let history = storage.history_get(ROOM, unix_epoch(0)).await.unwrap();
        assert_eq!(history, vec![p1]);

        let full_history = storage
            .history_get_entries(ROOM, unix_epoch(0))
            .await
            .unwrap();
        assert_history_without_timestamp(&full_history, &[Entry::start(p1)]);

        let speaker = storage.speaker_get(ROOM).await.unwrap();
        assert_eq!(speaker, Some(p1));

        assert_eq!(
            next,
            Some(StateMachineOutput::SpeakerUpdate(exchange::SpeakerUpdate {
                speaker: Some(p1),
                history: Some(vec![p1]),
                remaining: Some(vec![p2])
            }))
        );

        // Check non-allowed participant
        let next = select_next(storage, ROOM, &config, Some(p3), &mut rng).await;
        assert!(matches!(next, Err(Error::InvalidSelection)));

        let history = storage.history_get(ROOM, unix_epoch(0)).await.unwrap();
        assert_eq!(history, vec![p1]);

        let full_history = storage
            .history_get_entries(ROOM, unix_epoch(0))
            .await
            .unwrap();
        assert_history_without_timestamp(&full_history, &[Entry::start(p1)]);

        let speaker = storage.speaker_get(ROOM).await.unwrap();
        assert_eq!(speaker, Some(p1));

        // Check with nominee in history
        let next = select_next(storage, ROOM, &config, Some(p1), &mut rng).await;
        assert!(matches!(next, Err(Error::InvalidSelection)));

        let history = storage.history_get(ROOM, unix_epoch(0)).await.unwrap();
        assert_eq!(history, vec![p1]);

        let full_history = storage
            .history_get_entries(ROOM, unix_epoch(0))
            .await
            .unwrap();
        assert_history_without_timestamp(&full_history, &[Entry::start(p1)]);

        let speaker = storage.speaker_get(ROOM).await.unwrap();
        assert_eq!(speaker, Some(p1));
    }

    #[tokio::test]
    #[serial]
    async fn nomination_without_nomination_empty_allow_list_redis() {
        nomination_without_nomination_empty_allow_list(&mut setup_redis().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn nomination_without_nomination_empty_allow_list_memory() {
        nomination_without_nomination_empty_allow_list(&mut setup_memory().await).await;
    }

    /// Test next when selection_strategy is Nomination but no nomination was given and the allow_list
    /// is empty
    async fn nomination_without_nomination_empty_allow_list(storage: &mut dyn AutomodStorage) {
        let mut rng = rng();

        let config = StorageConfig {
            started: unix_epoch(0),
            issued_by: ParticipantId::nil(),
            parameter: Parameter {
                selection_strategy: SelectionStrategy::Nomination,
                show_list: false,
                consider_hand_raise: false,
                time_limit: None,
                allow_double_selection: false,
                animation_on_random: false,
                auto_append_on_join: false,
            },
        };

        let next = select_next(storage, ROOM, &config, None, &mut rng)
            .await
            .unwrap();

        let history = storage.history_get(ROOM, unix_epoch(0)).await.unwrap();
        assert_eq!(history, Vec::new());

        let full_history = storage
            .history_get_entries(ROOM, unix_epoch(0))
            .await
            .unwrap();
        assert!(full_history.is_empty());

        let speaker = storage.speaker_get(ROOM).await.unwrap();
        assert_eq!(speaker, None);

        assert_eq!(next, None);
    }

    #[tokio::test]
    #[serial]
    async fn nomination_without_nomination_with_allow_list_redis() {
        nomination_without_nomination_with_allow_list(&mut setup_redis().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn nomination_without_nomination_with_allow_list_memory() {
        nomination_without_nomination_with_allow_list(&mut setup_memory().await).await;
    }

    /// Test next when selection_strategy is Nomination but no nomination was given but the
    /// allow_list contains possible participants
    async fn nomination_without_nomination_with_allow_list(storage: &mut dyn AutomodStorage) {
        let mut rng = rng();

        let p1 = ParticipantId::from_u128(1);
        let p2 = ParticipantId::from_u128(2);
        let p3 = ParticipantId::from_u128(3);

        storage.allow_list_set(ROOM, &[p1, p2, p3]).await.unwrap();

        let config = StorageConfig {
            started: unix_epoch(0),
            issued_by: p1,
            parameter: Parameter {
                selection_strategy: SelectionStrategy::Nomination,
                show_list: false,
                consider_hand_raise: false,
                time_limit: None,
                allow_double_selection: false,
                animation_on_random: false,
                auto_append_on_join: false,
            },
        };

        // select_next with empty history
        let next = select_next(storage, ROOM, &config, None, &mut rng)
            .await
            .unwrap();
        assert_eq!(next, None);

        let history = storage.history_get(ROOM, unix_epoch(0)).await.unwrap();
        assert_eq!(history, Vec::new());

        let full_history = storage
            .history_get_entries(ROOM, unix_epoch(0))
            .await
            .unwrap();
        assert!(full_history.is_empty());

        let speaker = storage.speaker_get(ROOM).await.unwrap();
        assert_eq!(speaker, None);

        // Add current speaker
        state_machine::select_unchecked(storage, ROOM, &config, Some(p1))
            .await
            .unwrap();

        // select_next with non-empty history
        let next = select_next(storage, ROOM, &config, None, &mut rng)
            .await
            .unwrap()
            .unwrap();

        let next = match next {
            StateMachineOutput::SpeakerUpdate(speaker_update) => speaker_update,
            StateMachineOutput::StartAnimation(_) => panic!(),
        };

        assert!(next.speaker.is_none());
        assert_eq!(next.history.unwrap(), vec![p1]);
        let mut remaining = next.remaining.unwrap();
        remaining.sort();
        assert_eq!(remaining, vec![p1, p2, p3]);

        let history = storage.history_get(ROOM, unix_epoch(0)).await.unwrap();
        assert_eq!(history, vec![p1]);

        let full_history = storage
            .history_get_entries(ROOM, unix_epoch(0))
            .await
            .unwrap();
        assert_history_without_timestamp(&full_history, &[Entry::start(p1), Entry::stop(p1)]);

        let speaker = storage.speaker_get(ROOM).await.unwrap();
        assert_eq!(speaker, None);
    }

    #[tokio::test]
    #[serial]
    async fn select_next_with_none_redis() {
        select_next_with_none(&mut setup_redis().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn select_next_with_none_memory() {
        select_next_with_none(&mut setup_memory().await).await;
    }

    /// Test next when selection_strategy is None
    async fn select_next_with_none(storage: &mut dyn AutomodStorage) {
        let mut rng = rng();

        let p1 = ParticipantId::from_u128(1);

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

        let next = select_next(storage, ROOM, &config, None, &mut rng)
            .await
            .unwrap();

        let history = storage.history_get(ROOM, unix_epoch(0)).await.unwrap();
        assert_eq!(history, vec![]);

        let full_history = storage
            .history_get_entries(ROOM, unix_epoch(0))
            .await
            .unwrap();
        assert_history_without_timestamp(&full_history, &[]);

        let speaker = storage.speaker_get(ROOM).await.unwrap();
        assert_eq!(speaker, None);

        assert_eq!(next, None);

        // Add current speaker
        storage.history_add(ROOM, Entry::start(p1)).await.unwrap();
        storage.speaker_set(ROOM, p1).await.unwrap();

        // select_next with non-empty history
        let next = select_next(storage, ROOM, &config, None, &mut rng)
            .await
            .unwrap();

        let history = storage.history_get(ROOM, unix_epoch(0)).await.unwrap();
        assert_eq!(history, vec![p1]);

        let full_history = storage
            .history_get_entries(ROOM, unix_epoch(0))
            .await
            .unwrap();
        assert_history_without_timestamp(&full_history, &[Entry::start(p1), Entry::stop(p1)]);

        let speaker = storage.speaker_get(ROOM).await.unwrap();
        assert_eq!(speaker, None);

        assert_eq!(
            next,
            Some(StateMachineOutput::SpeakerUpdate(exchange::SpeakerUpdate {
                speaker: None,
                history: Some(vec![p1]),
                remaining: Some(vec![])
            }))
        );
    }

    #[tokio::test]
    #[serial]
    async fn select_next_with_playlist_redis() {
        select_next_with_playlist(&mut setup_redis().await).await
    }

    #[tokio::test]
    #[serial]
    async fn select_next_with_playlist_memory() {
        select_next_with_playlist(&mut setup_memory().await).await
    }

    /// Test next when selection_strategy is Playlist
    async fn select_next_with_playlist(storage: &mut dyn AutomodStorage) {
        let mut rng = rng();

        let p1 = ParticipantId::from_u128(1);
        let p2 = ParticipantId::from_u128(2);
        let p3 = ParticipantId::from_u128(3);

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

        // Test with empty playlist
        let next = select_next(storage, ROOM, &config, None, &mut rng)
            .await
            .unwrap();

        let history = storage.history_get(ROOM, unix_epoch(0)).await.unwrap();
        assert_eq!(history, vec![]);

        let full_history = storage
            .history_get_entries(ROOM, unix_epoch(0))
            .await
            .unwrap();
        assert_history_without_timestamp(&full_history, &[]);

        let speaker = storage.speaker_get(ROOM).await.unwrap();
        assert_eq!(speaker, None);

        assert_eq!(next, None);

        // Create playlist
        storage.playlist_set(ROOM, &[p2, p1, p3]).await.unwrap();

        // select_next with empty history and playlist
        let next = select_next(storage, ROOM, &config, None, &mut rng)
            .await
            .unwrap();

        let history = storage.history_get(ROOM, unix_epoch(0)).await.unwrap();
        assert_eq!(history, vec![p2]);

        let full_history = storage
            .history_get_entries(ROOM, unix_epoch(0))
            .await
            .unwrap();
        assert_history_without_timestamp(&full_history, &[Entry::start(p2)]);

        let speaker = storage.speaker_get(ROOM).await.unwrap();
        assert_eq!(speaker, Some(p2));

        assert_eq!(
            next,
            Some(StateMachineOutput::SpeakerUpdate(exchange::SpeakerUpdate {
                speaker: Some(p2),
                history: Some(vec![p2]),
                remaining: Some(vec![p1, p3])
            }))
        );

        // select_next with non-empty history and playlist
        let next = select_next(storage, ROOM, &config, None, &mut rng)
            .await
            .unwrap();

        let history = storage.history_get(ROOM, unix_epoch(0)).await.unwrap();
        assert_eq!(history, vec![p2, p1]);

        let full_history = storage
            .history_get_entries(ROOM, unix_epoch(0))
            .await
            .unwrap();
        assert_history_without_timestamp(
            &full_history,
            &[Entry::start(p2), Entry::stop(p2), Entry::start(p1)],
        );

        let speaker = storage.speaker_get(ROOM).await.unwrap();
        assert_eq!(speaker, Some(p1));

        assert_eq!(
            next,
            Some(StateMachineOutput::SpeakerUpdate(exchange::SpeakerUpdate {
                speaker: Some(p1),
                history: Some(vec![p2, p1]),
                remaining: Some(vec![p3])
            }))
        );

        // select_next with non-empty history and playlist, to drain playlist
        let next = select_next(storage, ROOM, &config, None, &mut rng)
            .await
            .unwrap();
        assert_eq!(
            next,
            Some(StateMachineOutput::SpeakerUpdate(exchange::SpeakerUpdate {
                speaker: Some(p3),
                history: Some(vec![p2, p1, p3]),
                remaining: Some(vec![])
            }))
        );

        // select_next with non-empty history and empty-playlist
        let next = select_next(storage, ROOM, &config, None, &mut rng)
            .await
            .unwrap();

        let history = storage.history_get(ROOM, unix_epoch(0)).await.unwrap();
        assert_eq!(history, vec![p2, p1, p3]);

        let full_history = storage
            .history_get_entries(ROOM, unix_epoch(0))
            .await
            .unwrap();
        assert_history_without_timestamp(
            &full_history,
            &[
                Entry::start(p2),
                Entry::stop(p2),
                Entry::start(p1),
                Entry::stop(p1),
                Entry::start(p3),
                Entry::stop(p3),
            ],
        );

        let speaker = storage.speaker_get(ROOM).await.unwrap();
        assert_eq!(speaker, None);

        assert_eq!(
            next,
            Some(StateMachineOutput::SpeakerUpdate(exchange::SpeakerUpdate {
                speaker: None,
                history: Some(vec![p2, p1, p3]),
                remaining: Some(vec![])
            }))
        );
    }
}
