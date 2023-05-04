// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{
    core::{BreakoutRoomId, ParticipantId, ParticipationKind, Timestamp},
    signaling::Role,
};

#[allow(unused_imports)]
use crate::imports::*;

/// Information about a participant in another breakout room
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ParticipantInOtherRoom {
    /// The id of the breakout room
    #[cfg_attr(feature = "serde", serde(default))]
    pub breakout_room: Option<BreakoutRoomId>,

    /// The id of the other participant
    pub id: ParticipantId,

    /// The display name of the other participant
    pub display_name: String,

    /// The role of the other participant
    pub role: Role,

    /// The URL to the avatar of the other participant
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub avatar_url: Option<String>,

    /// The participantion kind of the other participant
    pub participation_kind: ParticipationKind,

    /// The timestamp when the other participant joined
    pub joined_at: Timestamp,

    /// The timestamp when the other participant left
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub left_at: Option<Timestamp>,
}
