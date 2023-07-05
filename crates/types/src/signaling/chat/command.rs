// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `chat` namespace

use super::Scope;

#[allow(unused_imports)]
use crate::imports::*;

/// Send a chat message content with a specific scope
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SendMessage {
    /// The content of the message
    pub content: String,

    /// The scope of the message
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub scope: Scope,
}
