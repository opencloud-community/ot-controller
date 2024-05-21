// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod polls_storage;
mod redis;
mod volatile;

pub(crate) use polls_storage::PollsStorage;
// TODO: remove these re-exports once available in the PollsStorage trait
pub(crate) use redis::{del_results, del_state, list_add, list_members, poll_results, vote};

#[cfg(test)]
mod test_common {
    use std::time::Duration;

    use opentalk_signaling_core::SignalingRoomId;
    use opentalk_types::{
        core::Timestamp,
        signaling::polls::{state::PollsState, Choice, ChoiceId, PollId},
    };

    use super::PollsStorage;

    pub const ROOM: SignalingRoomId = SignalingRoomId::nil();

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
    }
}
