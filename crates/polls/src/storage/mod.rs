// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod polls_storage;
mod redis;
mod volatile;

pub(crate) use polls_storage::PollsStorage;

#[cfg(test)]
mod test_common {
    use std::{
        collections::{BTreeMap, BTreeSet},
        iter::repeat_with,
        time::Duration,
    };

    use opentalk_signaling_core::SignalingRoomId;
    use opentalk_types::signaling::polls::{state::PollsState, Choice, Item, PollId};
    use opentalk_types_common::time::Timestamp;
    use opentalk_types_signaling_polls::ChoiceId;

    use super::PollsStorage;

    pub const ROOM: SignalingRoomId = SignalingRoomId::nil();

    pub const CHOICE_1: ChoiceId = ChoiceId::from_u32(1u32);
    pub const CHOICE_2: ChoiceId = ChoiceId::from_u32(2u32);

    pub(super) async fn polls_state(storage: &mut dyn PollsStorage) {
        let polls_state = PollsState {
            id: PollId::nil(),
            topic: "Some poll".to_string(),
            live: true,
            multiple_choice: false,
            choices: vec![
                Choice {
                    id: ChoiceId::from(0u32),
                    content: "Choice A".to_string(),
                },
                Choice {
                    id: ChoiceId::from(1u32),
                    content: "Choice B".to_string(),
                },
            ],
            started: Timestamp::now(),
            duration: Duration::from_secs(10),
        };

        let mut polls_state_2 = polls_state.clone();
        polls_state_2.live = false;
        polls_state_2.topic = "Another poll".to_string();

        assert!(storage.get_polls_state(ROOM).await.unwrap().is_none());

        assert!(storage.set_polls_state(ROOM, &polls_state).await.unwrap());
        assert_eq!(
            storage.get_polls_state(ROOM).await.unwrap(),
            Some(polls_state.clone())
        );

        // Ensure that we don't set the polls state again if there is already one set
        assert!(!storage.set_polls_state(ROOM, &polls_state_2).await.unwrap());
        assert_eq!(
            storage.get_polls_state(ROOM).await.unwrap(),
            Some(polls_state.clone())
        );

        storage.delete_polls_state(ROOM).await.unwrap();
        assert_eq!(storage.get_polls_state(ROOM).await.unwrap(), None);
    }

    pub(super) async fn voting(storage: &mut dyn PollsStorage) {
        let polls_state = PollsState {
            id: PollId::nil(),
            topic: "Some poll".to_string(),
            live: true,
            multiple_choice: false,
            choices: vec![
                Choice {
                    id: CHOICE_1,
                    content: "Choice A".to_string(),
                },
                Choice {
                    id: CHOICE_2,
                    content: "Choice B".to_string(),
                },
            ],
            started: Timestamp::now(),
            duration: Duration::from_secs(10),
        };
        assert!(storage.get_polls_state(ROOM).await.unwrap().is_none());

        assert!(storage.set_polls_state(ROOM, &polls_state).await.unwrap());

        for _ in 0..4 {
            storage
                .vote(
                    ROOM,
                    polls_state.id,
                    &BTreeSet::from([]),
                    &BTreeSet::from([CHOICE_1]),
                )
                .await
                .unwrap();
        }

        let choices = storage.results(ROOM, polls_state.id).await.unwrap();
        assert_eq!(choices, BTreeMap::from([(CHOICE_1, 4)]));

        for _ in 0..2 {
            storage
                .vote(
                    ROOM,
                    polls_state.id,
                    &BTreeSet::from([CHOICE_1]),
                    &BTreeSet::from([CHOICE_2]),
                )
                .await
                .unwrap();
        }

        let results = storage.results(ROOM, polls_state.id).await.unwrap();
        assert_eq!(results, BTreeMap::from([(CHOICE_1, 2), (CHOICE_2, 2)]));

        let results = storage.poll_results(ROOM, &polls_state).await.unwrap();
        assert_eq!(
            &results,
            &[
                Item {
                    id: CHOICE_1,
                    count: 2
                },
                Item {
                    id: CHOICE_2,
                    count: 2
                }
            ]
        );
    }

    pub(super) async fn polls(storage: &mut dyn PollsStorage) {
        let poll_ids = Vec::from_iter(repeat_with(PollId::generate).take(4));
        for &id in &poll_ids {
            storage.add_poll_to_list(ROOM, id).await.unwrap();
        }

        assert_eq!(poll_ids, storage.poll_ids(ROOM).await.unwrap());

        storage.delete_poll_ids(ROOM).await.unwrap();

        assert!(storage.poll_ids(ROOM).await.unwrap().is_empty());
    }
}
