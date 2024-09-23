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
mod instance_id;

pub use call_in_info::CallInInfo;
pub use delete_email_invite_body::DeleteEmailInviteBody;
pub use delete_event_invite_path::DeleteEventInvitePath;
pub use delete_events_query::DeleteEventsQuery;
pub use delete_shared_folder_query::DeleteSharedFolderQuery;
pub use email_invite::EmailInvite;
pub use email_only_user::EmailOnlyUser;
pub use event_and_instance_id::EventAndInstanceId;
pub use instance_id::InstanceId;

/// The format string used for formatting UTC datetimes
const UTC_DT_FORMAT: &str = "%Y%m%dT%H%M%SZ";
