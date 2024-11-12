// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2
use opentalk_types_signaling::ParticipantId;

use crate::whisper_id::WhisperId;

/// Another set of participants was invited to the whisper group
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParticipantsInvited {
    /// The id of the whisper group
    pub whisper_id: WhisperId,
    /// The participants that were invited
    pub participant_ids: Vec<ParticipantId>,
}
