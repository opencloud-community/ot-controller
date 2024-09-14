// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to signaling events in the `meeting-notes` namespace

use opentalk_types_signaling_meeting_notes::event::{AccessUrl, PdfAsset};

#[allow(unused_imports)]
use crate::imports::*;

/// Events sent out by the `meeting-notes` module
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case", tag = "message")
)]
pub enum MeetingNotesEvent {
    /// An access url containing a write session
    WriteUrl(AccessUrl),

    /// An access url containing a readonly session
    ReadUrl(AccessUrl),

    /// Handle to the PDF asset
    PdfAsset(PdfAsset),

    /// An error happened when executing a `meeting-notes` command
    Error(Error),
}

impl From<PdfAsset> for MeetingNotesEvent {
    fn from(value: PdfAsset) -> Self {
        Self::PdfAsset(value)
    }
}

/// Errors from the `meeting-notes` module namespace
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case", tag = "error")
)]
pub enum Error {
    /// The requesting user has insufficient permissions for the operation
    InsufficientPermissions,
    /// The request contains invalid participant ids
    InvalidParticipantSelection,
    /// Is send when another instance just started initializing and etherpad is not available yet
    CurrentlyInitializing,
    /// The etherpad initialization failed
    FailedInitialization,
    /// The etherpad is not yet initailized
    NotInitialized,
    /// The requesting user has exceeded their storage
    StorageExceeded,
}

impl From<Error> for MeetingNotesEvent {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::{self, json};

    use super::*;

    #[test]
    fn write_url() {
        let expected = json!({
            "message": "write_url",
            "url": "http://localhost/auth_session?sessionID=s.session&padName=meeting_notes&groupID=g.group",
        });

        let message = MeetingNotesEvent::WriteUrl(AccessUrl {
            url:
                "http://localhost/auth_session?sessionID=s.session&padName=meeting_notes&groupID=g.group"
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

        let message = MeetingNotesEvent::ReadUrl(AccessUrl {
            url: "http://localhost:9001/auth_session?sessionID=s.session_id&padName=r.readonly_id"
                .into(),
        });

        let actual = serde_json::to_value(message).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn insufficient_permissions() {
        let expected = json!({"message": "error", "error": "insufficient_permissions"});

        let message = MeetingNotesEvent::Error(Error::InsufficientPermissions);

        let actual = serde_json::to_value(message).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn currently_initialization() {
        let expected = json!({"message": "error", "error": "failed_initialization"});

        let message = MeetingNotesEvent::Error(Error::FailedInitialization);

        let actual = serde_json::to_value(message).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn failed_initializing() {
        let expected = json!({"message": "error", "error": "currently_initializing"});

        let message = MeetingNotesEvent::Error(Error::CurrentlyInitializing);

        let actual = serde_json::to_value(message).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn invalid_participant_selection() {
        let expected = json!({"message": "error", "error": "invalid_participant_selection"});

        let message = MeetingNotesEvent::Error(Error::InvalidParticipantSelection);

        let actual = serde_json::to_value(message).unwrap();

        assert_eq!(expected, actual);
    }
}
