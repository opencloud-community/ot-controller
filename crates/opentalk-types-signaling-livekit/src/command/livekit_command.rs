// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;

/// The livekit command variants
#[derive(Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "snake_case", tag = "action")
)]
pub enum LiveKitCommand {
    /// Indicates that a new Access Token is requested
    CreateNewAccessToken,

    /// Force mutes participants
    ForceMute {
        /// The participants that should get muted
        participants: Vec<ParticipantId>,
    },

    /// Allows the specified participants to share their screens
    GrantScreenSharePermission {
        /// The participants that get granted screen sharing permissions
        participants: Vec<ParticipantId>,
    },

    /// Revokes the permission to share their screen
    RevokeScreenSharePermission {
        /// The participants
        participants: Vec<ParticipantId>,
    },

    /// Enables the microphone restriction state where only the participants that are part of the
    /// [`UnrestrictedParticipants::unrestricted_participants`] are allowed to unmute themselves. This will mute
    /// all participants who are not allowed to unmute themselves, but are currently not muted.
    EnableMicrophoneRestrictions(UnrestrictedParticipants),

    /// Disable the microphone restriction state which will allow all participants
    /// to unmute their microphone again.
    DisableMicrophoneRestrictions,

    /// Request a new livekit access token that cannot publish and is hidden to other participants
    RequestPopoutStreamAccessToken,
}

/// Request a number of participants to mute themselves
///
/// May only be processed if the issuer is a moderator
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnrestrictedParticipants {
    /// Participants that are still allowed to unmute
    pub unrestricted_participants: Vec<ParticipantId>,
}
