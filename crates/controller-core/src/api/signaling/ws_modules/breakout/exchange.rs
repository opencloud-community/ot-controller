// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::HashMap, time::SystemTime};

use opentalk_types::signaling::breakout::{
    AssociatedParticipantInOtherRoom, ParticipantInOtherRoom,
};
use opentalk_types_common::rooms::BreakoutRoomId;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use super::storage::BreakoutConfig;

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
