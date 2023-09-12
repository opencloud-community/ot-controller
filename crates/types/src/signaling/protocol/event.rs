// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to signaling events in the `protocol` namespace

use crate::core::AssetId;

#[allow(unused_imports)]
use crate::imports::*;

/// Events sent out by the `protocol` module
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case", tag = "message")
)]
pub enum ProtocolEvent {
    /// An access url containing a write session
    WriteUrl(AccessUrl),

    /// An access url containing a readonly session
    ReadUrl(AccessUrl),

    /// Handle to the PDF asset
    PdfAsset(PdfAsset),

    /// An error happened when executing a `protocol` command
    Error(Error),
}

/// The access URL to a specific data
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
pub struct AccessUrl {
    /// URL for the data
    pub url: String,
}

/// Handle to a PDF asset
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PdfAsset {
    /// The file name of the PDF asset
    pub filename: String,

    /// The asset id for the PDF asset
    pub asset_id: AssetId,
}

impl From<PdfAsset> for ProtocolEvent {
    fn from(value: PdfAsset) -> Self {
        Self::PdfAsset(value)
    }
}

/// Errors from the `protocol` module namespace
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
}

impl From<Error> for ProtocolEvent {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json;
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
