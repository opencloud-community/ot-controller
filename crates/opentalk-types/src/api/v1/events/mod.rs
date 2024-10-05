// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 events endpoints.

mod patch_event_body;
mod patch_event_instance_body;
mod post_events_body;

pub mod invites;

pub use patch_event_body::PatchEventBody;
pub use patch_event_instance_body::PatchEventInstanceBody;
pub use post_events_body::PostEventsBody;
