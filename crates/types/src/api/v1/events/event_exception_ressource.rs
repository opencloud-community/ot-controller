// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;
use crate::{
    api::v1::users::PublicUserProfile,
    core::{DateTimeTz, EventId, Timestamp},
};

use super::{EventAndInstanceId, EventStatus, EventType, InstanceId};

/// Event exception resource
///
/// Overrides event properties for a event recurrence. May only exist for events of type `recurring`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EventExceptionResource {
    /// Opaque ID of the exception
    pub id: EventAndInstanceId,

    /// ID of the event  the exception belongs to
    pub recurring_event_id: EventId,

    /// ID of the instance the exception overrides
    pub instance_id: InstanceId,

    /// Public user profile of the user which created the exception
    pub created_by: PublicUserProfile,

    /// Timestamp of the exceptions creation
    pub created_at: Timestamp,

    /// Public user profile of the user which last updated the exception
    pub updated_by: PublicUserProfile,

    /// Timestamp of the exceptions last update
    pub updated_at: Timestamp,

    /// Override the title of the instance
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub title: Option<String>,

    /// Override the description of the instance
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub description: Option<String>,

    /// Override the `is_all_day` property of the instance
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub is_all_day: Option<bool>,

    /// Override the `starts_at` time of the instance
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub starts_at: Option<DateTimeTz>,

    /// Override the `ends_at` time of the instance
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub ends_at: Option<DateTimeTz>,

    /// The `starts_at` of the instance this exception modifies. Used to match the exception the instance
    pub original_starts_at: DateTimeTz,

    /// Must always be `exception`
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub type_: EventType,

    /// Override the status of the event instance
    ///
    /// This can be used to cancel a occurrence of an event
    pub status: EventStatus,

    /// Can the current user edit this resource
    pub can_edit: bool,
}
