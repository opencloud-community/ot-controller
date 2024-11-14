// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the recording service.
//!
//! Please be aware that in contrast to the typical signaling modules, command
//! and event are reversed from the backend view.
//!
//! Usually an `event` is sent from the backend to the frontend. In this case,
//! a `command` is sent to the recording service, and an `event` is sent back to the
//! backend service.

mod recording_service_event;

pub use recording_service_event::RecordingServiceEvent;
