// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod livekit_storage;
mod redis;
mod volatile;

pub(crate) use livekit_storage::LivekitStorage;

#[cfg(test)]
mod test_common {

    use std::collections::BTreeSet;

    use opentalk_types_common::rooms;
    use opentalk_types_signaling::ParticipantId;
    use opentalk_types_signaling_livekit::MicrophoneRestrictionState;
    use pretty_assertions::assert_eq;

    use super::LivekitStorage;

    pub const ALICE: ParticipantId = ParticipantId::from_u128(0xbadcafe);

    pub(super) async fn force_mute(storage: &mut dyn LivekitStorage) {
        let room = rooms::RoomId::from_u128(123);
        assert_eq!(
            storage
                .get_microphone_restriction_state(room)
                .await
                .unwrap(),
            MicrophoneRestrictionState::Disabled
        );

        storage
            .set_microphone_restriction_allow_list(room, &[ALICE])
            .await
            .unwrap();

        assert_eq!(
            storage
                .get_microphone_restriction_state(room)
                .await
                .unwrap(),
            MicrophoneRestrictionState::Enabled {
                unrestricted_participants: BTreeSet::from_iter([ALICE])
            }
        );

        storage
            .set_microphone_restriction_allow_list(room, &[])
            .await
            .unwrap();

        assert_eq!(
            storage
                .get_microphone_restriction_state(room)
                .await
                .unwrap(),
            MicrophoneRestrictionState::Disabled
        );
    }
}
