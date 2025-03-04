// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use opentalk_types_common::time::Timestamp;
use opentalk_types_signaling::ParticipantId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Checkpoint {
    pub timestamp: Timestamp,
    pub presence: BTreeMap<ParticipantId, Timestamp>,
}
