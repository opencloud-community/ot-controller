// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the recording service.
//!
//! Please be aware that in contrast to the typical signaling modules, command
//! and event are reversed from the backend view.
//!
//! Usually a `command` is sent from the frontend to the backend. In this case,
//! a `command` is sent to the recording service, and an `event` is sent back to the
//! backend service.

mod recording_service_command;

pub use recording_service_command::RecordingServiceCommand;
