// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 events endpoints.

mod event_exception_resource;
mod event_instance;
mod event_invitee;
mod event_invitee_profile;
mod event_or_exception;
mod event_resource;
mod event_room_info;
mod event_status;
mod event_type;
mod get_event_instance_response_body;
mod get_event_instances_cursor_data;
mod get_event_instances_query;
mod get_event_instances_response_body;
mod get_event_query;
mod get_events_cursor_data;
mod get_events_query;
mod patch_email_invite_body;
mod patch_event_body;
mod patch_event_instance_body;
mod patch_event_query;
mod patch_invite_body;
mod post_event_invite_body;
mod post_event_invite_query;
mod post_events_body;
mod public_invite_user_profile;
mod put_shared_folder_query;
mod streaming_target_options_query;
mod user_invite;

pub mod invites;

pub use event_exception_resource::EventExceptionResource;
pub use event_instance::EventInstance;
pub use event_invitee::EventInvitee;
pub use event_invitee_profile::EventInviteeProfile;
pub use event_or_exception::EventOrException;
pub use event_resource::EventResource;
pub use event_room_info::EventRoomInfo;
pub use event_status::EventStatus;
pub use event_type::EventType;
pub use get_event_instance_response_body::GetEventInstanceResponseBody;
pub use get_event_instances_cursor_data::GetEventInstancesCursorData;
pub use get_event_instances_query::GetEventInstancesQuery;
pub use get_event_instances_response_body::GetEventInstancesResponseBody;
pub use get_event_query::GetEventQuery;
pub use get_events_cursor_data::GetEventsCursorData;
pub use get_events_query::GetEventsQuery;
pub use patch_email_invite_body::PatchEmailInviteBody;
pub use patch_event_body::PatchEventBody;
pub use patch_event_instance_body::PatchEventInstanceBody;
pub use patch_event_query::PatchEventQuery;
pub use patch_invite_body::PatchInviteBody;
pub use post_event_invite_body::PostEventInviteBody;
pub use post_event_invite_query::PostEventInviteQuery;
pub use post_events_body::PostEventsBody;
pub use public_invite_user_profile::PublicInviteUserProfile;
pub use put_shared_folder_query::PutSharedFolderQuery;
pub use streaming_target_options_query::StreamingTargetOptionsQuery;
pub use user_invite::UserInvite;
