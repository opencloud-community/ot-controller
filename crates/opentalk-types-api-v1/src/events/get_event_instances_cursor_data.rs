// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::CursorData;

/// Data stored inside the `GET /events/{event_id}/instances` query cursor
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GetEventInstancesCursorData {
    /// Page number
    pub page: i64,
}

impl CursorData for GetEventInstancesCursorData {
    const SCHEMA_CURSOR_TYPE_NAME: &'static str = "GetEventInstancesCursor";
}
