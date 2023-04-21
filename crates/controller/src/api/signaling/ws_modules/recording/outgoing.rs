// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Serialize;
use types::signaling::recording::event::{Error, Started, Stopped};

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum RecordingEvent {
    Started(Started),
    Stopped(Stopped),
    Error(Error),
}
