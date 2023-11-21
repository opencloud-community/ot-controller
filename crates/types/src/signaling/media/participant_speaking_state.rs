// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::core::ParticipantId;

#[allow(unused_imports)]
use crate::imports::*;

use super::SpeakingState;

/// The state of a recent or current speaker in the conference
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ParticipantSpeakingState {
    /// The participant id of the speaker
    pub participant: ParticipantId,

    /// Information about the speaking state
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub speaker: SpeakingState,
}
