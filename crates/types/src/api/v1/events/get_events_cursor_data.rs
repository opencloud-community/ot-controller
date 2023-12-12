// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::core::{EventId, Timestamp};
#[allow(unused_imports)]
use crate::imports::*;

/// Data stored inside the `GET /events` query cursor
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GetEventsCursorData {
    /// Last event in the list
    pub event_id: EventId,

    /// last event created at
    pub event_created_at: Timestamp,

    /// Last event starts_at
    pub event_starts_at: Option<Timestamp>,
}
