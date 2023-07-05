// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `chat` namespace

use crate::core::ParticipantId;

#[allow(unused_imports)]
use crate::imports::*;

/// The chat history was cleared
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HistoryCleared {
    /// ID of the participant that cleared chat history
    pub issued_by: ParticipantId,
}

/// Errors from the `chat` module namespace
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "error", rename_all = "snake_case")
)]
pub enum Error {
    /// Request while chat is disabled
    ChatDisabled,

    /// Request user has insufficient permissions
    InsufficientPermissions,
}
