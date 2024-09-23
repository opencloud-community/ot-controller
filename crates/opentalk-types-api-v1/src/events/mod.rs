// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 events endpoints.

mod call_in_info;
mod delete_email_invite_body;
mod delete_event_invite_path;

pub use call_in_info::CallInInfo;
pub use delete_email_invite_body::DeleteEmailInviteBody;
pub use delete_event_invite_path::DeleteEventInvitePath;
