// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 events endpoints.

mod call_in_info;
mod email_only_user;
mod event_and_instance_id;
mod event_room_info;
mod event_status;
mod event_type;
mod get_events_cursor_data;
mod instance_id;
mod post_events_body;
mod public_invite_user_profile;

pub use call_in_info::CallInInfo;
pub use email_only_user::EmailOnlyUser;
pub use event_and_instance_id::EventAndInstanceId;
pub use event_room_info::EventRoomInfo;
pub use event_status::EventStatus;
pub use event_type::EventType;
pub use get_events_cursor_data::GetEventsCursorData;
pub use instance_id::InstanceId;
pub use post_events_body::PostEventsBody;
pub use public_invite_user_profile::PublicInviteUserProfile;

const UTC_DT_FORMAT: &str = "%Y%m%dT%H%M%SZ";
