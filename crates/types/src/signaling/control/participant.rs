// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashMap;

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
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub module_data: HashMap<String, serde_json::Value>,
}

impl Participant {
    /// Gets the inner module of a Participant
    #[cfg(feature = "serde")]
    pub fn get_module<T: DeserializeOwned>(
        &self,
        namespace: &str,
    ) -> Result<Option<T>, serde_json::Error> {
        self.module_data
            .get(namespace)
            .map(|m| serde_json::from_value(m.clone()))
            .transpose()
    }
}
