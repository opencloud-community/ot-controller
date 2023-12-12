// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 events endpoints.

mod call_in_info;
mod delete_email_invite_body;
mod delete_event_invite_path;
mod delete_event_query;
mod delete_query;
mod email_only_user;
mod event_and_instance_id;
mod event_exception_ressource;
mod event_instance;
mod event_instance_path;
mod event_instance_query;
mod event_invitee;
mod event_invitee_profile;
mod event_or_exception;
mod event_ressource;
mod event_room_info;
mod event_status;
mod event_type;
mod get_event_instances_cursor_data;
mod get_event_instances_query;
mod get_event_query;
mod get_events_cursor_data;
mod get_events_query;
mod instance_id;
mod patch_event_body;
mod patch_event_instance_body;
mod patch_event_query;
mod patch_invite_body;
mod post_event_invite_body;
mod post_event_invite_query;
mod post_events_body;
mod public_invite_user_profile;
mod user_invite;

pub use call_in_info::CallInInfo;
pub use delete_email_invite_body::DeleteEmailInviteBody;
pub use delete_event_invite_path::DeleteEventInvitePath;
pub use delete_event_query::DeleteEventQuery;
pub use delete_query::DeleteQuery;
pub use email_only_user::EmailOnlyUser;
pub use event_and_instance_id::EventAndInstanceId;
pub use event_exception_ressource::EventExceptionResource;
pub use event_instance::EventInstance;
pub use event_instance_path::EventInstancePath;
pub use event_instance_query::EventInstanceQuery;
pub use event_invitee::EventInvitee;
pub use event_invitee_profile::EventInviteeProfile;
pub use event_or_exception::EventOrException;
pub use event_ressource::EventResource;
pub use event_room_info::EventRoomInfo;
pub use event_status::EventStatus;
pub use event_type::EventType;
pub use get_event_instances_cursor_data::GetEventInstancesCursorData;
pub use get_event_instances_query::GetEventInstancesQuery;
pub use get_event_query::GetEventQuery;
pub use get_events_cursor_data::GetEventsCursorData;
pub use get_events_query::GetEventsQuery;
pub use instance_id::InstanceId;
pub use patch_event_body::PatchEventBody;
pub use patch_event_instance_body::PatchEventInstanceBody;
pub use patch_event_query::PatchEventQuery;
pub use patch_invite_body::PatchInviteBody;
pub use post_event_invite_body::PostEventInviteBody;
pub use post_event_invite_query::PostEventInviteQuery;
pub use post_events_body::PostEventsBody;
pub use public_invite_user_profile::PublicInviteUserProfile;
pub use user_invite::UserInvite;

const UTC_DT_FORMAT: &str = "%Y%m%dT%H%M%SZ";
