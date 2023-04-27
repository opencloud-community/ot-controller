// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to signaling events in the `media` namespace

use crate::imports::*;

/// The direction of a media link
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum LinkDirection {
    /// Upstream direction
    Upstream,

    /// Downstream direction
    Downstream,
}
