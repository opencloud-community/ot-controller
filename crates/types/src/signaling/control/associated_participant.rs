// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{core::ParticipantId, imports::*};

/// AssociatedParticipant represents a participant in the same meeting
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AssociatedParticipant {
    /// The participant id for the associated participant
    pub id: ParticipantId,
}
