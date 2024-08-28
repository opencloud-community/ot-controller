// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;
use crate::{
    api::v1::{events::GetEventInstancesCursorData, Cursor},
    core::Timestamp,
};

/// Query parameters for the `GET /events/{event_id}/instances` endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::IntoParams))]
pub struct GetEventInstancesQuery {
    /// Maximum number of invitees to include inside the event
    #[cfg_attr(feature = "serde", serde(default))]
    pub invitees_max: i64,
    /// Minimum time of the event instances
    pub time_min: Option<Timestamp>,
    /// Maximum time of the event instances
    pub time_max: Option<Timestamp>,
    /// How many events to return per page
    pub per_page: Option<i64>,
    /// Cursor token to get the next page of events
    #[cfg_attr(feature="utoipa", param(schema_with = build_cursor_schema))]
    pub after: Option<Cursor<GetEventInstancesCursorData>>,
}

#[cfg(feature = "utoipa")]
fn build_cursor_schema() -> impl Into<utoipa::openapi::RefOr<utoipa::openapi::Schema>> {
    use utoipa::{openapi::Ref, ToSchema as _};
    Ref::from_schema_name(Cursor::<GetEventInstancesCursorData>::schema().0)
}
