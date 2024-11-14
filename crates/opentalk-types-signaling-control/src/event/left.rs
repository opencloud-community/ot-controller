// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::{AssociatedParticipant, LeaveReason};

/// A participant left.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Left {
    /// The participant that left
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub id: AssociatedParticipant,

    /// The reason as to why the participant left
    pub reason: LeaveReason,
}
