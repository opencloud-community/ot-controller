// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_types_common::streaming::StreamingTargetId;

/// Data for the `stop` streaming command
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StopStreaming {
    /// Id of the to be stopped stream
    pub target_ids: BTreeSet<StreamingTargetId>,
}
