// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_api_v1::Cursor;
use opentalk_types_common::{events::invites::EventInviteStatus, time::Timestamp};

use super::GetEventsCursorData;
#[cfg(feature = "serde")]
use crate::api::v1::utils::comma_separated;
#[allow(unused_imports)]
use crate::imports::*;

/// Path query parameters of the `GET /events` endpoint
///
/// Allows for customization in the search for events
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::IntoParams))]
pub struct GetEventsQuery {
    /// Optional minimum time in which the event happens
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub time_min: Option<Timestamp>,

    /// Optional maximum time in which the event happens
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub time_max: Option<Timestamp>,

    /// Maximum number of invitees to return inside the event resource
    ///
    /// Default value is 0
    #[cfg_attr(feature = "serde", serde(default))]
    pub invitees_max: u32,

    /// Return only favorite events
    #[cfg_attr(feature = "serde", serde(default))]
    pub favorites: bool,

    /// Filter the events by invite status
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            skip_serializing_if = "Vec::is_empty",
            with = "comma_separated",
        )
    )]
    pub invite_status: Vec<EventInviteStatus>,

    /// How many events to return per page
    pub per_page: Option<i64>,

    /// Cursor token to get the next page of events
    ///
    /// Returned by the endpoint if the maximum number of events per page has been hit
    #[cfg_attr(feature = "utoipa", param(inline))]
    pub after: Option<Cursor<GetEventsCursorData>>,

    /// Only get events that are either marked as adhoc or non-adhoc
    ///
    /// If present, all adhoc events will be returned when `true`, all non-adhoc
    /// events will be returned when `false`. If not present, all events will
    /// be returned regardless of their `adhoc` flag value.
    pub adhoc: Option<bool>,

    /// Only get events that are either time-independent or time-dependent
    ///
    /// If present, all time-independent events will be returned when `true`,
    /// all time-dependent events will be returned when `false`. If absent,
    /// all events will be returned regardless of their time dependency.
    pub time_independent: Option<bool>,
}
