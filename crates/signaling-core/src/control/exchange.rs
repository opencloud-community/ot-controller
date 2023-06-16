// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::SignalingRoomId;

use serde::{Deserialize, Serialize};
use types::core::{ParticipantId, RoomId, UserId};

/// Control messages sent between controller modules to communicate changes inside a room
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Message {
    /// Participant with the given id joined the current room
    Joined(ParticipantId),

    /// Participant with the given id left the current room
    Left(ParticipantId),

    /// Participant with the given id updated its status
    Update(ParticipantId),

    /// Participant with the given id, was accepted into the room
    ///
    /// This message is only sent to the participant to be accepted and published by the `moderation` module.
    /// The control module only handles the joining into the waiting room and joining the actual room.
    Accepted(ParticipantId),

    SetModeratorStatus(bool),

    EnableRaiseHands {
        issued_by: ParticipantId,
    },
    DisableRaiseHands {
        issued_by: ParticipantId,
    },

    ResetRaisedHands {
        issued_by: ParticipantId,
    },

    RoomDeleted,
}

// ==== Current room routing-keys

/// Create a routing key addressing all participants by their user-id in the specified room
pub fn current_room_by_user_id(room_id: SignalingRoomId, id: UserId) -> String {
    format!("room={room_id}:user={id}")
}

/// Create a routing key addressing a participant by id in the specified room
pub fn current_room_by_participant_id(room_id: SignalingRoomId, id: ParticipantId) -> String {
    format!("room={room_id}:participant={id}")
}

/// Create a routing key addressing all participants in the specified room
pub fn current_room_all_participants(room_id: SignalingRoomId) -> String {
    format!("room={room_id}:participants")
}

// ==== Global room routing-keys

/// Create a routing key addressing all participants by their user-id in the specified room and it's breakout rooms
pub fn global_room_by_user_id(room_id: RoomId, id: UserId) -> String {
    format!("global_room={room_id}:user={id}")
}

/// Create a routing key addressing a participant by id in the specified room and it's breakout rooms
pub fn global_room_by_participant_id(room_id: RoomId, id: ParticipantId) -> String {
    format!("global_room={room_id}:participant={id}")
}

/// Create a routing key addressing all participants in the specified room and it's breakout rooms
pub fn global_room_all_participants(room_id: RoomId) -> String {
    format!("global_room={room_id}:participants")
}
