// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 events endpoints.

mod event_or_exception;
mod event_resource;
mod get_event_instance_response_body;
mod get_event_instances_cursor_data;
mod get_event_instances_query;
mod get_event_instances_response_body;
mod get_events_cursor_data;
mod get_events_query;
mod patch_event_body;
mod patch_event_instance_body;
mod post_events_body;

pub mod invites;

pub use event_or_exception::EventOrException;
pub use event_resource::EventResource;
pub use get_event_instance_response_body::GetEventInstanceResponseBody;
pub use get_event_instances_cursor_data::GetEventInstancesCursorData;
pub use get_event_instances_query::GetEventInstancesQuery;
pub use get_event_instances_response_body::GetEventInstancesResponseBody;
pub use get_events_cursor_data::GetEventsCursorData;
pub use get_events_query::GetEventsQuery;
pub use patch_event_body::PatchEventBody;
pub use patch_event_instance_body::PatchEventInstanceBody;
pub use post_events_body::PostEventsBody;
