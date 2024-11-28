// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2
use opentalk_types_signaling::ParticipantId;

/// An error related to the subroom audio functionality
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "error", rename_all = "snake_case")
)]

pub enum Error {
    /// The provided whisper id does not exist
    InvalidWhisperId,
    /// The participant has already accepted the group invite
    AlreadyAccepted,
    /// The requesting participant has insufficient permissions
    InsufficientPermissions,
    /// The list of invited participant is empty
    EmptyParticipantList,
    /// The targeted participants do not exist
    InvalidParticipantTargets {
        /// A list of invalid participants
        participant_ids: Vec<ParticipantId>,
    },
    /// The livekit server is unavailable
    LivekitUnavailable,
    /// The requesting participant has no access to the whisper group
    NotInvited,
}
