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
        /// The participants
        participants: Vec<ParticipantId>,
    },
    /// Allows the permission to share their screen
    GrantScreenSharePermission {
        /// The participants
        participants: Vec<ParticipantId>,
    },
    /// Revokes the permission to share their screen
    RevokeScreenSharePermission {
        /// The participants
        participants: Vec<ParticipantId>,
    },
}
