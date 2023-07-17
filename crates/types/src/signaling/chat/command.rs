// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `chat` namespace

use super::Scope;

use crate::core::Timestamp;
#[allow(unused_imports)]
use crate::imports::*;

/// Commands for the `chat` namespace
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "action", rename_all = "snake_case")
)]
pub enum ChatCommand {
    /// Enable chat messaging
    EnableChat,

    /// Disable chat messaging
    DisableChat,

    /// Send chat message
    SendMessage(SendMessage),

    /// Clear chat history
    ClearHistory,

    /// Set last seen timestamp
    SetLastSeenTimestamp {
        /// Scope of the timestamp
        #[cfg_attr(feature = "serde", serde(flatten))]
        scope: Scope,

        /// Last seen timestamp
        timestamp: Timestamp,
    },
}

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

#[cfg(test)]
mod test {
    use crate::core::{GroupName, ParticipantId};

    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn user_private_message() {
        let json = json!({
            "action": "send_message",
            "scope": "private",
            "target": "00000000-0000-0000-0000-000000000000",
            "content": "Hello Bob!"
        });

        let msg: ChatCommand = serde_json::from_value(json).unwrap();

        if let ChatCommand::SendMessage(SendMessage { content, scope }) = msg {
            assert_eq!(scope, Scope::Private(ParticipantId::nil()));
            assert_eq!(content, "Hello Bob!");
        } else {
            panic!()
        }
    }

    #[test]
    fn user_group_message() {
        let json = json!({
            "action": "send_message",
            "scope": "group",
            "target": "management",
            "content": "Hello managers!"
        });

        let msg: ChatCommand = serde_json::from_value(json).unwrap();

        if let ChatCommand::SendMessage(SendMessage { content, scope }) = msg {
            assert_eq!(
                scope,
                Scope::Group(GroupName::from("management".to_owned()))
            );
            assert_eq!(content, "Hello managers!");
        } else {
            panic!()
        }
    }

    #[test]
    fn user_room_message() {
        let json = json!({
            "action": "send_message",
            "scope": "global",
            "content": "Hello all!"
        });

        let msg: ChatCommand = serde_json::from_value(json).unwrap();

        if let ChatCommand::SendMessage(SendMessage { content, scope }) = msg {
            assert_eq!(scope, Scope::Global);
            assert_eq!(content, "Hello all!");
        } else {
            panic!()
        }
    }
}
