// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::storage::BreakoutConfig;
use super::AssocParticipantInOtherRoom;
use crate::api::signaling::BreakoutRoomId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use types::{core::ParticipantId, signaling::breakout::ParticipantInOtherRoom};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Message {
    Start(Start),
    Stop,

    Joined(ParticipantInOtherRoom),
    Left(AssocParticipantInOtherRoom),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Start {
    pub config: BreakoutConfig,
    pub started: SystemTime,
    pub assignments: HashMap<ParticipantId, BreakoutRoomId>,
}
