// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;
use crate::{
    api::v1::users::PublicUserProfile,
    common::shared_folder::SharedFolder,
    core::{DateTimeTz, EventId, EventInviteStatus, Timestamp},
};

use super::{EventAndInstanceId, EventInvitee, EventRoomInfo, EventStatus, EventType, InstanceId};

/// Event instance resource
///
/// An event instance is an occurrence of an recurring event
///
/// Exceptions for the instance are always already applied
///
/// For infos on undocumented fields see [`EventResource`]
///
/// [`EventResource`]: ../event_ressource/struct.EventResource.html
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EventInstance {
    /// Opaque id of the event instance resource
    pub id: EventAndInstanceId,

    /// ID of the recurring event this instance belongs to
    pub recurring_event_id: EventId,

    /// Opaque id of the instance
    pub instance_id: InstanceId,

    /// Public user profile of the user which created the event
    pub created_by: PublicUserProfile,

    /// Timestamp of the event creation
    pub created_at: Timestamp,

    /// Public user profile of the user which last updated the event
    /// or created the exception which modified the instance
    pub updated_by: PublicUserProfile,

    /// Timestamp of the last update
    pub updated_at: Timestamp,

    /// Title of the event
    pub title: String,
    /// Description of the event
    pub description: String,
    /// All information about the room the event takes place in
    pub room: EventRoomInfo,
    /// Flag which indicates if `invitees` contains all invites as far as known to the application
    pub invitees_truncated: bool,
    /// List of event invitees and their invite status. Might not be complete, see `invite_truncated`
    pub invitees: Vec<EventInvitee>,
    /// Flag indicating whether the event is all-day
    pub is_all_day: bool,
    /// Start time of the event.
    pub starts_at: DateTimeTz,
    /// End time of the event.
    pub ends_at: DateTimeTz,

    /// Must always be `instance`
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub type_: EventType,
    /// The invite status of the current user for this event
    pub status: EventStatus,
    /// Is this event in the current user's favorite list?
    pub invite_status: EventInviteStatus,
    /// Flag to indicate if the event is a favorite of the current user
    pub is_favorite: bool,
    /// Fkag to indicate if the current user can edit the event
    pub can_edit: bool,

    /// Information about the shared folder for the event
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub shared_folder: Option<SharedFolder>,
}
