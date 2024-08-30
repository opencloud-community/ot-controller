// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Data types for handling events.

pub mod invites;

mod event_id;
mod meeting_details;

pub use event_id::EventId;
pub use meeting_details::MeetingDetails;
