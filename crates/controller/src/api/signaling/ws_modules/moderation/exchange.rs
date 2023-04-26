// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};
use types::{core::ParticipantId, signaling::moderation::KickScope};

/// Control messages sent between controller modules to communicate changes inside a room
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Message {
    Kicked(ParticipantId),
    Banned(ParticipantId),
    Debriefed {
        kick_scope: KickScope,
        issued_by: ParticipantId,
    },
    JoinedWaitingRoom(ParticipantId),
    LeftWaitingRoom(ParticipantId),
    WaitingRoomEnableUpdated,
}
