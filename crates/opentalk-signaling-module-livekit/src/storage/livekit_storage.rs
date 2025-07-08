// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{
    SignalingModuleError, control::storage::ControlStorageParticipantSet,
};
use opentalk_types_common::rooms::RoomId;
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_livekit::MicrophoneRestrictionState;

#[async_trait]
pub(crate) trait LivekitStorage: ControlStorageParticipantSet + Send {
    async fn set_microphone_restriction_allow_list(
        &mut self,
        room: RoomId,
        participants: &[ParticipantId],
    ) -> Result<(), SignalingModuleError>;

    async fn clear_microphone_restriction(
        &mut self,
        room: RoomId,
    ) -> Result<(), SignalingModuleError>;

    async fn get_microphone_restriction_state(
        &mut self,
        room: RoomId,
    ) -> Result<MicrophoneRestrictionState, SignalingModuleError>;
}
