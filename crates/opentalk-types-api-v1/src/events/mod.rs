// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 events endpoints.

mod call_in_info;
mod delete_email_invite_body;
mod delete_event_invite_path;
mod delete_events_query;
mod delete_shared_folder_query;
mod email_invite;
mod email_only_user;
mod event_and_instance_id;
mod event_exception_resource;
mod event_instance;
mod event_instance_path;
mod event_instance_query;
mod event_invitee;
mod event_invitee_profile;
mod event_options_query;
mod event_or_exception;
mod event_resource;
mod event_room_info;
mod event_status;
mod event_type;
mod get_event_instance_response_body;
mod get_event_instances_response_body;
mod get_event_query;
mod instance_id;
mod patch_email_invite_body;
mod patch_event_query;
mod patch_invite_body;
mod post_event_invite_body;
mod post_event_invite_query;
mod public_invite_user_profile;
mod put_shared_folder_query;
mod streaming_target_options_query;
mod user_invite;

pub use call_in_info::CallInInfo;
pub use delete_email_invite_body::DeleteEmailInviteBody;
pub use delete_event_invite_path::DeleteEventInvitePath;
pub use delete_events_query::DeleteEventsQuery;
pub use delete_shared_folder_query::DeleteSharedFolderQuery;
pub use email_invite::EmailInvite;
pub use email_only_user::EmailOnlyUser;
pub use event_and_instance_id::EventAndInstanceId;
pub use event_exception_resource::EventExceptionResource;
pub use event_instance::EventInstance;
pub use event_instance_path::EventInstancePath;
pub use event_instance_query::EventInstanceQuery;
pub use event_invitee::EventInvitee;
pub use event_invitee_profile::EventInviteeProfile;
pub use event_options_query::EventOptionsQuery;
pub use event_or_exception::EventOrException;
pub use event_resource::EventResource;
pub use event_room_info::EventRoomInfo;
pub use event_status::EventStatus;
pub use event_type::EventType;
pub use get_event_instance_response_body::GetEventInstanceResponseBody;
pub use get_event_instances_response_body::GetEventInstancesResponseBody;
pub use get_event_query::GetEventQuery;
pub use instance_id::InstanceId;
pub use patch_email_invite_body::PatchEmailInviteBody;
pub use patch_event_query::PatchEventQuery;
pub use patch_invite_body::PatchInviteBody;
pub use post_event_invite_body::PostEventInviteBody;
pub use post_event_invite_query::PostEventInviteQuery;
pub use public_invite_user_profile::PublicInviteUserProfile;
pub use put_shared_folder_query::PutSharedFolderQuery;
pub use streaming_target_options_query::StreamingTargetOptionsQuery;
pub use user_invite::UserInvite;

/// The format string used for formatting UTC datetimes
const UTC_DT_FORMAT: &str = "%Y%m%dT%H%M%SZ";
