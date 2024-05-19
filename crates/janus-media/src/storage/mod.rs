// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

pub mod presenter;
pub mod speaker;

mod media_storage;
mod redis;
mod volatile;

pub(crate) use media_storage::MediaStorage;

#[cfg(test)]
mod test_common {
    use opentalk_signaling_core::SignalingRoomId;
    use opentalk_types::{
        core::ParticipantId,
        signaling::media::{MediaSessionState, ParticipantMediaState},
    };
    use pretty_assertions::assert_eq;

    use super::*;

    pub const ROOM: SignalingRoomId = SignalingRoomId::nil();
    pub const BOB: ParticipantId = ParticipantId::from_u128(0xdeadbeef);

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
}
