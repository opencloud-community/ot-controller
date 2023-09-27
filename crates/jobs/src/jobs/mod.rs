// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! # Jobs that can be run in the OpenTalk job execution system

mod adhoc_event_cleanup;
mod event_cleanup;
mod invite_cleanup;
mod self_check;

pub use adhoc_event_cleanup::AdhocEventCleanup;
pub use event_cleanup::EventCleanup;
pub use invite_cleanup::InviteCleanup;
pub use self_check::SelfCheck;
