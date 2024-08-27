// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{events::EventId, utils::ExampleData};

#[allow(unused_imports)]
use crate::imports::*;
use crate::{api::v1::cursor::CursorData, core::Timestamp};

/// Data stored inside the `GET /events` query cursor
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(GetEventsCursorData::example_data()))
)]
pub struct GetEventsCursorData {
    /// Last event in the list
    pub event_id: EventId,

    /// last event created at
    pub event_created_at: Timestamp,

    /// Last event starts_at
    pub event_starts_at: Option<Timestamp>,
}

impl ExampleData for GetEventsCursorData {
    fn example_data() -> Self {
        Self {
            event_id: EventId::example_data(),
            event_created_at: Timestamp::example_data(),
            event_starts_at: None,
        }
    }
}

impl CursorData for GetEventsCursorData {
    const SCHEMA_CURSOR_TYPE_NAME: &'static str = "GetEventsCursor";
}
