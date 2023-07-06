// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `chat` namespace

use crate::core::ParticipantId;

#[allow(unused_imports)]
use crate::imports::*;

use super::{MessageId, Scope};

/// A message
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MessageSent {
    /// Id of the message
    pub id: MessageId,

    /// Sender of the message
    pub source: ParticipantId,

    /// Content of the message
    pub content: String,

    /// Scope of the message
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub scope: Scope,
}

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
