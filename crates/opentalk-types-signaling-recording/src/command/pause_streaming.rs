// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_types_common::streaming::StreamingTargetId;

/// Data for the `pause` streaming command
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PauseStreaming {
    /// Id of the to be paused stream
    pub target_ids: BTreeSet<StreamingTargetId>,
}
