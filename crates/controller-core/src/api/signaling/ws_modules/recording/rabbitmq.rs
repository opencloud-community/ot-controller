// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types::core::RoomId;
use opentalk_types_common::rooms::BreakoutRoomId;
use serde::Serialize;

/// Message sent to the recording service instructing it to record the given room
#[derive(Debug, Clone, Serialize)]
pub struct InitializeRecorder {
    pub room: RoomId,
    pub breakout: Option<BreakoutRoomId>,
}
