// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod redis;
mod subroom_audio_storage;
mod volatile;

pub(crate) use subroom_audio_storage::SubroomAudioStorage;

#[cfg(test)]
mod test_common {

    use std::collections::BTreeMap;

    use opentalk_signaling_core::SignalingRoomId;
    use opentalk_types_signaling::ParticipantId;
    use opentalk_types_signaling_subroom_audio::{state::WhisperState, whisper_id::WhisperId};

    use super::SubroomAudioStorage;

    pub const ROOM: SignalingRoomId = SignalingRoomId::nil();
    pub const WHISPER_ID: WhisperId = WhisperId::nil();

    pub const PARTICIPANT_A: ParticipantId = ParticipantId::from_u128(0);
    pub const PARTICIPANT_B: ParticipantId = ParticipantId::from_u128(1);
    pub const PARTICIPANT_C: ParticipantId = ParticipantId::from_u128(2);

    pub const WHISPER_PARTICIPANT_A: (ParticipantId, WhisperState) =
        (PARTICIPANT_A, WhisperState::Creator);

    pub const WHISPER_PARTICIPANT_B: (ParticipantId, WhisperState) =
        (PARTICIPANT_B, WhisperState::Invited);

    pub const WHISPER_PARTICIPANT_C: (ParticipantId, WhisperState) =
        (PARTICIPANT_C, WhisperState::Invited);

    pub(crate) async fn create_group(storage: &mut dyn SubroomAudioStorage) {
        storage
            .create_whisper_group(
                ROOM,
                WHISPER_ID,
                &BTreeMap::from([WHISPER_PARTICIPANT_A, WHISPER_PARTICIPANT_B]),
            )
            .await
            .unwrap();

        let participants = storage.get_whisper_group(ROOM, WHISPER_ID).await.unwrap();

        assert_eq!(
            &participants,
            &BTreeMap::from([WHISPER_PARTICIPANT_A, WHISPER_PARTICIPANT_B])
        )
    }

    pub(crate) async fn add_participant(storage: &mut dyn SubroomAudioStorage) {
        storage
            .create_whisper_group(ROOM, WHISPER_ID, &BTreeMap::from([WHISPER_PARTICIPANT_A]))
            .await
            .unwrap();

        storage
            .add_participants(
                ROOM,
                WHISPER_ID,
                &BTreeMap::from([WHISPER_PARTICIPANT_B, WHISPER_PARTICIPANT_C]),
            )
            .await
            .unwrap();

        let participant_btree_map = storage.get_whisper_group(ROOM, WHISPER_ID).await.unwrap();

        assert_eq!(
            &participant_btree_map,
            &BTreeMap::from([
                WHISPER_PARTICIPANT_A,
                WHISPER_PARTICIPANT_B,
                WHISPER_PARTICIPANT_C
            ])
        )
    }

    pub(crate) async fn update_participant(storage: &mut dyn SubroomAudioStorage) {
        storage
            .create_whisper_group(
                ROOM,
                WHISPER_ID,
                &BTreeMap::from([
                    WHISPER_PARTICIPANT_A,
                    WHISPER_PARTICIPANT_B,
                    WHISPER_PARTICIPANT_C,
                ]),
            )
            .await
            .unwrap();

        let new_state = WhisperState::Accepted;

        storage
            .update_participant_state(ROOM, WHISPER_ID, PARTICIPANT_C, new_state)
            .await
            .unwrap();

        let participants = storage.get_whisper_group(ROOM, WHISPER_ID).await.unwrap();

        assert_eq!(
            &participants,
            &BTreeMap::from([
                WHISPER_PARTICIPANT_A,
                WHISPER_PARTICIPANT_B,
                (PARTICIPANT_C, new_state)
            ])
        )
    }

    pub(crate) async fn remove_participant(storage: &mut dyn SubroomAudioStorage) {
        storage
            .create_whisper_group(
                ROOM,
                WHISPER_ID,
                &BTreeMap::from([
                    WHISPER_PARTICIPANT_A,
                    WHISPER_PARTICIPANT_B,
                    WHISPER_PARTICIPANT_C,
                ]),
            )
            .await
            .unwrap();

        let group_deleted = storage
            .remove_participant(ROOM, WHISPER_ID, PARTICIPANT_C)
            .await
            .unwrap();
        assert!(!group_deleted);
        let group = storage.get_whisper_group(ROOM, WHISPER_ID).await.unwrap();

        assert_eq!(
            &group,
            &BTreeMap::from([WHISPER_PARTICIPANT_A, WHISPER_PARTICIPANT_B])
        );

        let group_deleted = storage
            .remove_participant(ROOM, WHISPER_ID, PARTICIPANT_B)
            .await
            .unwrap();
        assert!(!group_deleted);
        let group = storage.get_whisper_group(ROOM, WHISPER_ID).await.unwrap();
        assert_eq!(&group, &BTreeMap::from([WHISPER_PARTICIPANT_A]));

        let group_deleted = storage
            .remove_participant(ROOM, WHISPER_ID, PARTICIPANT_A)
            .await
            .unwrap();
        assert!(group_deleted);

        match storage.get_whisper_group(ROOM, WHISPER_ID).await {
            Ok(participants) => {
                assert!(participants.is_empty());
            }
            Err(err) => panic!("Expected valid result with empty participants, got error: {err}"),
        }
    }

    pub(crate) async fn manage_groups(storage: &mut dyn SubroomAudioStorage) {
        let whisper_group_a = WhisperId::from_u128(0);
        let whisper_group_b = WhisperId::from_u128(1);

        storage
            .create_whisper_group(
                ROOM,
                whisper_group_a,
                &BTreeMap::from([WHISPER_PARTICIPANT_A]),
            )
            .await
            .unwrap();

        storage
            .create_whisper_group(
                ROOM,
                whisper_group_b,
                &BTreeMap::from([WHISPER_PARTICIPANT_A]),
            )
            .await
            .unwrap();

        let mut groups = storage.get_all_whisper_group_ids(ROOM).await.unwrap();
        groups.sort();
        assert_eq!(&groups, &vec![whisper_group_a, whisper_group_b]);

        storage
            .delete_whisper_group(ROOM, whisper_group_b)
            .await
            .unwrap();
        let groups = storage.get_all_whisper_group_ids(ROOM).await.unwrap();
        assert_eq!(&groups, &vec![whisper_group_a]);

        storage
            .delete_whisper_group(ROOM, whisper_group_a)
            .await
            .unwrap();
        let groups = storage.get_all_whisper_group_ids(ROOM).await.unwrap();
        assert!(groups.is_empty());
    }
}
