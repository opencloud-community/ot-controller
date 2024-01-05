// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::core::ParticipantId;

#[allow(unused_imports)]
use crate::imports::*;

/// Status information about a participant
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Participant {
    /// The id of the participant
    pub id: ParticipantId,

    /// Module data for the participant
    #[cfg(feature = "serde")]
    #[serde(flatten)]
    pub module_data: crate::signaling::ModulePeerData,
}

impl Participant {
    /// Gets the inner module data of a Participant
    #[cfg(feature = "serde")]
    pub fn get_module<T: SignalingModulePeerFrontendData>(
        &self,
    ) -> Result<Option<T>, serde_json::Error> {
        self.module_data.get::<T>()
    }
}
