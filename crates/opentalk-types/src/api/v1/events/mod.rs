// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 events endpoints.

mod get_events_cursor_data;
mod get_events_query;
mod patch_event_body;
mod patch_event_instance_body;
mod post_events_body;

pub mod invites;

pub use get_events_cursor_data::GetEventsCursorData;
pub use get_events_query::GetEventsQuery;
pub use patch_event_body::PatchEventBody;
pub use patch_event_instance_body::PatchEventInstanceBody;
pub use post_events_body::PostEventsBody;
