// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Serialize;
use types::signaling::protocol::event::{Error, PdfAsset};

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "message")]
pub enum ProtocolEvent {
    /// An access url containing a write session
    WriteUrl(AccessUrl),
    /// An access url containing a readonly session
    ReadUrl(AccessUrl),
    PdfAsset(PdfAsset),
    Error(Error),
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AccessUrl {
    pub url: String,
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn write_url() {
        let expected = json!({
            "message": "write_url",
            "url": "http://localhost/auth_session?sessionID=s.session&padName=protocol&groupID=g.group",
        });

        let message = ProtocolEvent::WriteUrl(AccessUrl {
            url:
                "http://localhost/auth_session?sessionID=s.session&padName=protocol&groupID=g.group"
                    .into(),
        });

        let actual = serde_json::to_value(message).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn read_url() {
        let expected = json!({
            "message": "read_url",
            "url": "http://localhost:9001/auth_session?sessionID=s.session_id&padName=r.readonly_id",
        });

        let message = ProtocolEvent::ReadUrl(AccessUrl {
            url: "http://localhost:9001/auth_session?sessionID=s.session_id&padName=r.readonly_id"
                .into(),
        });

        let actual = serde_json::to_value(message).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn insufficient_permissions() {
        let expected = json!({"message": "error", "error": "insufficient_permissions"});

        let message = ProtocolEvent::Error(Error::InsufficientPermissions);

        let actual = serde_json::to_value(message).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn currently_initialization() {
        let expected = json!({"message": "error", "error": "failed_initialization"});

        let message = ProtocolEvent::Error(Error::FailedInitialization);

        let actual = serde_json::to_value(message).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn failed_initializing() {
        let expected = json!({"message": "error", "error": "currently_initializing"});

        let message = ProtocolEvent::Error(Error::CurrentlyInitializing);

        let actual = serde_json::to_value(message).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn invalid_participant_selection() {
        let expected = json!({"message": "error", "error": "invalid_participant_selection"});

        let message = ProtocolEvent::Error(Error::InvalidParticipantSelection);

        let actual = serde_json::to_value(message).unwrap();

        assert_eq!(expected, actual);
    }
}
