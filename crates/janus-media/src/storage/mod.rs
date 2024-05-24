// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod media_storage;
mod redis;
mod volatile;

pub(crate) use media_storage::MediaStorage;
//TODO:(a.weiche) remove this once refactor is done
pub(crate) use redis::{delete_publisher_info, get_publisher_info, set_publisher_info};

#[cfg(test)]
mod test_common {
    use opentalk_signaling_core::SignalingRoomId;
    use opentalk_types::{
        core::{ParticipantId, Timestamp},
        signaling::media::{
            MediaSessionState, ParticipantMediaState, ParticipantSpeakingState, SpeakingState,
        },
    };
    use pretty_assertions::assert_eq;

    use super::MediaStorage;
    use crate::mcu::McuId;

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

        assert_eq!(
            storage
                .get_speaking_state_multiple_participants(ROOM, &[BOB, ALICE])
                .await
                .unwrap(),
            vec![
                ParticipantSpeakingState {
                    participant: BOB,
                    speaker: SpeakingState {
                        is_speaking: true,
                        updated_at: bob_started_speaking_at
                    }
                },
                ParticipantSpeakingState {
                    participant: ALICE,
                    speaker: SpeakingState {
                        is_speaking: false,
                        updated_at: alice_started_speaking_at
                    }
                },
            ]
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

        assert_eq!(
            storage
                .get_speaking_state_multiple_participants(ROOM, &[BOB, ALICE])
                .await
                .unwrap(),
            vec![ParticipantSpeakingState {
                participant: ALICE,
                speaker: SpeakingState {
                    is_speaking: false,
                    updated_at: alice_started_speaking_at
                }
            }]
        );

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

        assert_eq!(
            storage
                .get_speaking_state_multiple_participants(ROOM, &[BOB, ALICE])
                .await
                .unwrap(),
            vec![]
        );
    }

    pub(super) async fn mcu_load(storage: &mut dyn MediaStorage) {
        let id = McuId::new("to_janus", "janus_exchange", "from_janus");
        let a = (id.clone(), Some(1));
        let b = (id.clone(), Some(2));
        let c = (id.clone(), Some(3));

        storage.initialize_mcu_load(a.0.clone(), a.1).await.unwrap();
        storage.initialize_mcu_load(b.0.clone(), b.1).await.unwrap();
        storage.initialize_mcu_load(c.0.clone(), c.1).await.unwrap();

        assert_eq!(
            storage.get_mcus_sorted_by_load().await.unwrap(),
            vec![a.clone(), b.clone(), c.clone()]
        );

        // increase load on `a`, so it gets sorted last
        storage.increase_mcu_load(a.0.clone(), a.1).await.unwrap();
        storage.increase_mcu_load(a.0.clone(), a.1).await.unwrap();
        assert_eq!(
            storage.get_mcus_sorted_by_load().await.unwrap(),
            vec![b.clone(), c.clone(), a.clone()]
        );

        // increase load on `c` even higher, so that it gets sorted after `a` now
        storage.increase_mcu_load(c.0.clone(), c.1).await.unwrap();
        assert_eq!(
            storage.get_mcus_sorted_by_load().await.unwrap(),
            vec![b.clone(), c.clone(), a.clone()]
        );
        storage.increase_mcu_load(c.0.clone(), c.1).await.unwrap();
        assert_eq!(
            storage.get_mcus_sorted_by_load().await.unwrap(),
            vec![b.clone(), a.clone(), c.clone()]
        );

        // decrease load on `a` back to `0`, so it gets sorted first again
        storage.decrease_mcu_load(a.0.clone(), a.1).await.unwrap();
        storage.decrease_mcu_load(a.0.clone(), a.1).await.unwrap();
        assert_eq!(
            storage.get_mcus_sorted_by_load().await.unwrap(),
            vec![a.clone(), b.clone(), c.clone()]
        );
    }
}
