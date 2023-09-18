// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Common types related to event

use crate::core::EventId;

#[allow(unused_imports)]
use crate::imports::*;

/// Information about an event
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EventInfo {
    /// The id of the event
    pub id: EventId,
    /// The title of the event
    pub title: String,
    /// True if the event was created ad-hoc
    pub is_adhoc: bool,
}
