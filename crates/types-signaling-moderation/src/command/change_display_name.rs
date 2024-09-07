// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;

/// Change the display name of the targeted guest
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChangeDisplayName {
    /// The new display name
    pub new_name: String,

    /// The participant that will have their name changed
    pub target: ParticipantId,
}
