// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;

use super::Error;
use crate::{command::UnrestrictedParticipants, Credentials};

/// The events emitted for livekit
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "message", rename_all = "snake_case")
)]
pub enum LiveKitEvent {
    /// The credentials for a client to use livekit
    Credentials(Credentials),

    /// The moderator enabled the microphone-restriction-state. Only participants listed in
    /// [`UnrestrictedParticipants::unrestricted_participants`] are able to unmute themselves.
    MicrophoneRestrictionsEnabled(UnrestrictedParticipants),

    /// The moderator disabled the microphone-restriction-state.
    /// Participants are allowed to unmute themselves again.
    MicrophoneRestrictionsDisabled,

    /// The moderator has force muted the participant.
    ForceMuted {
        /// The moderator who issued the force mute command.
        moderator: ParticipantId,
    },

    /// The last message couldn't be processed since an unexpected error occurred.
    Error(Error),
}
