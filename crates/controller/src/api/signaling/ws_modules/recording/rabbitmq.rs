// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Serialize;
use types::core::{BreakoutRoomId, RoomId};

/// Message sent to the recording service instructing it to record the given room
#[derive(Debug, Serialize)]
pub struct StartRecording {
    pub room: RoomId,
    pub breakout: Option<BreakoutRoomId>,
}
