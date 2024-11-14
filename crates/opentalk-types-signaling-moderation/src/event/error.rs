// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Error from the `moderation` module namespace
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "error", rename_all = "snake_case")
)]
pub enum Error {
    /// Cannot ban a guest participant
    CannotBanGuest,
    /// Cannot send the room owner to the waiting room
    CannotSendRoomOwnerToWaitingRoom,
    /// Cannot change the display name of registered users
    CannotChangeNameOfRegisteredUsers,
    /// Invalid display name
    InvalidDisplayName,
    /// Insufficient permissions to perform a command
    InsufficientPermissions,
}
