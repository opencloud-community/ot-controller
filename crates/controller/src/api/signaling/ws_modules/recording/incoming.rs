// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;
use types::signaling::recording::command::SetConsent;

use super::RecordingId;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum RecordingCommand {
    Start,
    Stop(Stop),
    SetConsent(SetConsent),
}

#[derive(Debug, Deserialize)]
pub struct Stop {
    pub recording_id: RecordingId,
}
