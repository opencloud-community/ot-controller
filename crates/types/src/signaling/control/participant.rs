// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashMap;

use crate::{core::ParticipantId, imports::*};

/// Status information about a participant
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Participant {
    /// The id of the participant
    pub id: ParticipantId,

    /// Module data for the participant
    #[serde(flatten)]
    pub module_data: HashMap<String, serde_json::Value>,
}
