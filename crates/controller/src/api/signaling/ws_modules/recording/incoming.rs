// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;
use types::signaling::recording::command::{SetConsent, Stop};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum RecordingCommand {
    Start,
    Stop(Stop),
    SetConsent(SetConsent),
}
