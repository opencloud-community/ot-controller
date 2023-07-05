// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};
use types::{
    core::ParticipantId,
    signaling::chat::event::{Error, HistoryCleared},
};

use crate::{MessageId, Scope};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum ChatEvent {
    ChatEnabled(ChatEnabled),
    ChatDisabled(ChatDisabled),
    MessageSent(MessageSent),
    HistoryCleared(HistoryCleared),
    Error(Error),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ChatEnabled {
    pub issued_by: ParticipantId,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ChatDisabled {
    pub issued_by: ParticipantId,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MessageSent {
    pub id: MessageId,
    pub source: ParticipantId,
    pub content: String,
    #[serde(flatten)]
    pub scope: Scope,
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use types::core::GroupName;

    #[test]
    fn global_serialize() {
        let produced = serde_json::to_value(ChatEvent::MessageSent(MessageSent {
            id: MessageId::nil(),
            source: ParticipantId::nil(),
            content: "Hello All!".to_string(),
            scope: Scope::Global,
        }))
        .unwrap();

        let expected = json!({
            "message": "message_sent",
            "id": "00000000-0000-0000-0000-000000000000",
            "source": "00000000-0000-0000-0000-000000000000",
            "content": "Hello All!",
            "scope": "global"
        });

        assert_eq!(expected, produced);
    }

    #[test]
    fn group_serialize() {
        let produced = serde_json::to_value(ChatEvent::MessageSent(MessageSent {
            id: MessageId::nil(),
            source: ParticipantId::nil(),
            content: "Hello managers!".to_string(),
            scope: Scope::Group(GroupName::from("management".to_owned())),
        }))
        .unwrap();
        let expected = json!({
            "message":"message_sent",
            "id":"00000000-0000-0000-0000-000000000000",
            "source":"00000000-0000-0000-0000-000000000000",
            "content":"Hello managers!",
            "scope":"group",
            "target":"management",
        });
        assert_eq!(expected, produced);
    }

    #[test]
    fn private_serialize() {
        let produced = serde_json::to_value(ChatEvent::MessageSent(MessageSent {
            id: MessageId::nil(),
            source: ParticipantId::nil(),
            content: "Hello All!".to_string(),
            scope: Scope::Private(ParticipantId::from_u128(1)),
        }))
        .unwrap();

        let expected = json!({
            "message": "message_sent",
            "id": "00000000-0000-0000-0000-000000000000",
            "source": "00000000-0000-0000-0000-000000000000",
            "content": "Hello All!",
            "scope": "private",
            "target": "00000000-0000-0000-0000-000000000001",
        });
        assert_eq!(expected, produced);
    }

    #[test]
    fn error_serialize() {
        let produced = serde_json::to_value(ChatEvent::Error(Error::ChatDisabled)).unwrap();
        let expected = json!({
            "message": "error",
            "error": "chat_disabled",
        });
        assert_eq!(expected, produced);
    }
}
