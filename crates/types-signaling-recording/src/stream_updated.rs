// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::streaming::StreamingTargetId;

use crate::StreamStatus;

/// A streaming target has been updated
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StreamUpdated {
    /// The target id
    pub target_id: StreamingTargetId,
    /// The status of the specified stream target
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub status: StreamStatus,
}
