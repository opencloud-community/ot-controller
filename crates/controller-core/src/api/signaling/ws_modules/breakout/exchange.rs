// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::storage::BreakoutConfig;
use opentalk_types::{
    core::{BreakoutRoomId, ParticipantId},
    signaling::breakout::{AssociatedParticipantInOtherRoom, ParticipantInOtherRoom},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Message {
    Start(Start),
    Stop,

    Joined(ParticipantInOtherRoom),
    Left(AssociatedParticipantInOtherRoom),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Start {
    pub config: BreakoutConfig,
    pub started: SystemTime,
    pub assignments: HashMap<ParticipantId, BreakoutRoomId>,
}
