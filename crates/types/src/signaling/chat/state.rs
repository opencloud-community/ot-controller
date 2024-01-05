// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling state for the `chat` namespace

use std::collections::HashMap;

use crate::core::{GroupName, ParticipantId, Timestamp};

#[allow(unused_imports)]
use crate::imports::*;

use super::{MessageId, Scope};

/// The state of the `chat` module
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChatState {
    /// Is the chat module enabled
    pub enabled: bool,

    /// Chat history for the room
    pub room_history: Vec<StoredMessage>,

    /// All group chat history in the room
    pub groups_history: Vec<GroupHistory>,

    /// All private chat history in the room
    pub private_history: Vec<PrivateHistory>,

    /// Timestamp for last time someone read a message
    pub last_seen_timestamp_global: Option<Timestamp>,

    /// Timestamp for last time someone read a private message
    pub last_seen_timestamps_private: HashMap<ParticipantId, Timestamp>,

    /// Timestamp for last time someone read a group message
    pub last_seen_timestamps_group: HashMap<GroupName, Timestamp>,
}

#[cfg(feature = "serde")]
impl SignalingModuleFrontendData for ChatState {
    const NAMESPACE: Option<&'static str> = Some(super::NAMESPACE);
}

/// Group chat history
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GroupHistory {
    /// Name of the group
    pub name: GroupName,

    /// Group chat history
    pub history: Vec<StoredMessage>,
}

/// Private chat history
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PrivateHistory {
    /// Private chat correspondent
    pub correspondent: ParticipantId,

    /// Private chat history
    pub history: Vec<StoredMessage>,
}

/// Message type stores in redis
///
/// This needs to have a inner timestamp.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "redis",
    derive(ToRedisArgs, FromRedisValue),
    to_redis_args(serde),
    from_redis_value(serde)
)]
pub struct StoredMessage {
    /// ID of message
    pub id: MessageId,

    /// ID of the participant who sent the message
    pub source: ParticipantId,

    /// Timestamp of when the message was sent
    pub timestamp: Timestamp,

    /// Content of the message
    pub content: String,

    /// Scope of the message
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub scope: Scope,
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::DateTime;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use std::str::FromStr;

    #[test]
    fn server_message() {
        let expected = json!({
            "id":"00000000-0000-0000-0000-000000000000",
            "source":"00000000-0000-0000-0000-000000000000",
            "timestamp":"2021-06-24T14:00:11.873753715Z",
            "content":"Hello All!",
            "scope":"global",
        });

        let produced = serde_json::to_value(StoredMessage {
            id: MessageId::nil(),
            source: ParticipantId::nil(),
            timestamp: DateTime::from_str("2021-06-24T14:00:11.873753715Z")
                .unwrap()
                .into(),
            content: "Hello All!".to_string(),
            scope: Scope::Global,
        })
        .unwrap();

        assert_eq!(expected, produced);
    }
}
