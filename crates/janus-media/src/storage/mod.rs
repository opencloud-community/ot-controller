// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

pub mod speaker;

mod media_storage;
mod redis;
mod volatile;

pub(crate) use media_storage::MediaStorage;

#[cfg(test)]
mod test_common {
    use opentalk_signaling_core::SignalingRoomId;
    use opentalk_types::{
        core::{ParticipantId, Timestamp},
        signaling::media::{MediaSessionState, ParticipantMediaState, SpeakingState},
    };
    use pretty_assertions::assert_eq;

    use super::MediaStorage;

    pub const ROOM: SignalingRoomId = SignalingRoomId::nil();
    pub const BOB: ParticipantId = ParticipantId::from_u128(0xdeadbeef);
    pub const ALICE: ParticipantId = ParticipantId::from_u128(0xbadcafe);

    pub(super) async fn media_state(storage: &mut dyn MediaStorage) {
        let bob_media_state = ParticipantMediaState {
            video: Some(MediaSessionState {
                video: true,
                audio: false,
            }),
            screen: None,
        };
        assert!(storage.get_media_state(ROOM, BOB).await.unwrap().is_none());

        storage
            .set_media_state(ROOM, BOB, &bob_media_state)
            .await
            .unwrap();

        assert_eq!(
            storage.get_media_state(ROOM, BOB).await.unwrap(),
            Some(bob_media_state)
        );

        storage.delete_media_state(ROOM, BOB).await.unwrap();

        assert!(storage.get_media_state(ROOM, BOB).await.unwrap().is_none());
    }

    pub(super) async fn presenter(storage: &mut dyn MediaStorage) {
        assert!(!storage.is_presenter(ROOM, BOB).await.unwrap());

        storage.add_presenter(ROOM, BOB).await.unwrap();

        assert!(storage.is_presenter(ROOM, BOB).await.unwrap());

        storage.remove_presenter(ROOM, BOB).await.unwrap();

        assert!(!storage.is_presenter(ROOM, BOB).await.unwrap());

        storage.add_presenter(ROOM, BOB).await.unwrap();
        storage.add_presenter(ROOM, ALICE).await.unwrap();

        storage.clear_presenters(ROOM).await.unwrap();
        assert!(!storage.is_presenter(ROOM, BOB).await.unwrap());
        assert!(!storage.is_presenter(ROOM, ALICE).await.unwrap());
    }

    pub(super) async fn speaking_state(storage: &mut dyn MediaStorage) {
        assert!(storage
            .get_speaking_state(ROOM, BOB)
            .await
            .unwrap()
            .is_none());

        let bob_started_speaking_at = Timestamp::now();
        storage
            .set_speaking_state(ROOM, BOB, true, bob_started_speaking_at)
            .await
            .unwrap();

        assert_eq!(
            storage.get_speaking_state(ROOM, BOB).await.unwrap(),
            Some(SpeakingState {
                is_speaking: true,
                updated_at: bob_started_speaking_at
            })
        );

        let alice_started_speaking_at = Timestamp::now();
        storage
            .set_speaking_state(ROOM, ALICE, false, alice_started_speaking_at)
            .await
            .unwrap();

        assert_eq!(
            storage.get_speaking_state(ROOM, ALICE).await.unwrap(),
            Some(SpeakingState {
                is_speaking: false,
                updated_at: alice_started_speaking_at
            })
        );

        storage.delete_speaking_state(ROOM, BOB).await.unwrap();

        assert!(storage
            .get_speaking_state(ROOM, BOB)
            .await
            .unwrap()
            .is_none());
        assert!(storage
            .get_speaking_state(ROOM, ALICE)
            .await
            .unwrap()
            .is_some());

        storage
            .set_speaking_state(ROOM, BOB, false, bob_started_speaking_at)
            .await
            .unwrap();

        storage
            .delete_speaking_state_multiple_participants(ROOM, &[ALICE, BOB])
            .await
            .unwrap();

        assert!(storage
            .get_speaking_state(ROOM, BOB)
            .await
            .unwrap()
            .is_none());
        assert!(storage
            .get_speaking_state(ROOM, ALICE)
            .await
            .unwrap()
            .is_none());
    }
}
