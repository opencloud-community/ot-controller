// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_moderation::{event::DisplayNameChanged, KickScope};
use serde::{Deserialize, Serialize};

/// Control messages sent between controller modules to communicate changes inside a room
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Message {
    Kicked(ParticipantId),
    Banned(ParticipantId),
    SentToWaitingRoom(ParticipantId),
    Debriefed {
        kick_scope: KickScope,
        issued_by: ParticipantId,
    },
    DisplayNameChanged(DisplayNameChanged),
    JoinedWaitingRoom(ParticipantId),
    LeftWaitingRoom(ParticipantId),
    WaitingRoomEnableUpdated,
}
