// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

/// Path query for the `PATCH /events/{event_id}/{instance_id}` endpoint
#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EventInstanceQuery {
    /// Maximum number of invitees to return inside the event instance resource
    ///
    /// Default: 0
    #[cfg_attr(feature = "serde", serde(default))]
    pub invitees_max: i64,
}
