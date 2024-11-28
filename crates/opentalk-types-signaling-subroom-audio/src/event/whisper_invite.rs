// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;

use super::WhisperGroupOutgoing;

/// An invite to a whisper group
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct WhisperInvite {
    /// The issuer of the invite
    pub issuer: ParticipantId,
    /// The whisper group
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub group: WhisperGroupOutgoing,
}
