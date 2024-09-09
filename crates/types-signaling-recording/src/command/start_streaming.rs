// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_types_common::streaming::StreamingTargetId;

/// Data for the `start` streaming command
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StartStreaming {
    /// Id of the to be started stream
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeSet::is_empty")
    )]
    pub target_ids: BTreeSet<StreamingTargetId>,
}
