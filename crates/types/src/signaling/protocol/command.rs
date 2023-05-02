// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `recording` namespace

use crate::{core::ParticipantId, imports::*};

/// Give a list of participants write access to the protocol
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
pub struct ParticipantSelection {
    /// The targeted participants
    pub participant_ids: Vec<ParticipantId>,
}
