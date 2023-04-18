// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to signaling events in the `control` namespace

use crate::imports::*;

/// The reason for blocking a participant from joining a meeting
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "reason", rename_all = "snake_case")
)]
pub enum JoinBlockedReason {
    /// The participant limit for the meeting's tariff has been reached
    ParticipantLimitReached,
}

/// Errors from the `control` module namespace
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "error", rename_all = "snake_case")
)]
pub enum Error {
    /// Payload sent to the `control` module had the wrong JSON format
    InvalidJson,

    /// Attempted to send data to a module namespace that does not exist
    InvalidNamespace,

    /// The chosen user name does not meet the requirements
    InvalidUsername,

    /// Participant attempted to join while already participating in the meeting
    AlreadyJoined,

    /// Attempted to perform a command on a participant that has not yet joined the meeting
    NotYetJoined,

    /// A participant attempted to join the meeting who is neither in the waiting room nor accepted
    NotAcceptedOrNotInWaitingRoom,

    /// Attempted to raise hand while handraising is disabled for the meeting
    RaiseHandsDisabled,

    /// Attempted to perform a command which requires more permissions
    InsufficientPermissions,

    /// Attempted to grant or revoke moderation permissions to the room owner who implicitly has these permissions anyway
    TargetIsRoomOwner,

    /// An issued command requires no further actions
    NothingToDo,
}
