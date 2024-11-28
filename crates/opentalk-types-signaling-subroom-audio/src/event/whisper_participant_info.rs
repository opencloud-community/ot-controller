// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2
use opentalk_types_signaling::ParticipantId;

use crate::whisper_id::WhisperId;

/// A participant in a whisper group
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WhisperParticipantInfo {
    /// The id of the whisper group
    pub whisper_id: WhisperId,
    /// The participant
    pub participant_id: ParticipantId,
}
