// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

/// Path query parameters for the `GET /events/{event_id}` endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GetEventQuery {
    /// Maximum number of invitees to return inside the event resource
    ///
    /// Default: 0
    #[cfg_attr(feature = "serde", serde(default))]
    pub invitees_max: i64,
}
