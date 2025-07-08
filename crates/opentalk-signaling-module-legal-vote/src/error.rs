// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_database::DatabaseError;
use opentalk_signaling_core::{ObjectStorageError, SignalingModuleError, assets::AssetError};
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_legal_vote::event::{ErrorKind as TypesErrorKind, GuestParticipants};
use snafu::Snafu;

/// A legal vote error
#[derive(Debug, Snafu)]
pub(crate) enum LegalVoteError {
    /// A vote error caused by invalid requests
    #[snafu(transparent)]
    Vote { source: ErrorKind },

    /// A fatal error
    /// A general fatal error has occurred (bug or IO)
    #[snafu(whatever)]
    Fatal {
        message: String,

        #[snafu(source(from(Box<dyn std::error::Error+ Send + Sync>, Some)))]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

/// A non critical vote error caused by invalid requests
#[derive(Debug, Snafu, PartialEq)]
pub(crate) enum ErrorKind {
    #[snafu(display("A vote is already active"))]
    VoteAlreadyActive,
    #[snafu(display("No vote is currently taking place"))]
    NoVoteActive,
    #[snafu(display("The provided vote id is invalid"))]
    InvalidVoteId,
    #[snafu(display("The given allowlist contains guests: {guests:?}"))]
    AllowlistContainsGuests { guests: Vec<ParticipantId> },
    #[snafu(display("Failed to set or get permissions"))]
    PermissionError,
    #[snafu(display("The requesting user has insufficient permissions"))]
    InsufficientPermissions,
    #[snafu(display("The requesting user has exceeded their storage"))]
    StorageExceeded,
}

impl From<ErrorKind> for TypesErrorKind {
    fn from(value: ErrorKind) -> Self {
        match value {
            ErrorKind::VoteAlreadyActive => TypesErrorKind::VoteAlreadyActive,
            ErrorKind::NoVoteActive => TypesErrorKind::NoVoteActive,
            ErrorKind::InvalidVoteId => TypesErrorKind::InvalidVoteId,
            ErrorKind::AllowlistContainsGuests { guests } => {
                TypesErrorKind::AllowlistContainsGuests(GuestParticipants { guests })
            }
            ErrorKind::PermissionError => TypesErrorKind::PermissionError,
            ErrorKind::InsufficientPermissions => TypesErrorKind::InsufficientPermissions,
            ErrorKind::StorageExceeded => TypesErrorKind::StorageExceeded,
        }
    }
}

impl From<DatabaseError> for LegalVoteError {
    fn from(source: DatabaseError) -> Self {
        Self::Fatal {
            message: "Database error".to_string(),
            source: Some(Box::new(source)),
        }
    }
}

impl From<SignalingModuleError> for LegalVoteError {
    fn from(source: SignalingModuleError) -> Self {
        Self::Fatal {
            message: "Signaling module error".into(),
            source: Some(source.into()),
        }
    }
}

impl From<AssetError> for LegalVoteError {
    fn from(source: AssetError) -> Self {
        match source {
            AssetError::AssetStorageExceeded => Self::Vote {
                source: ErrorKind::StorageExceeded,
            },
            source => Self::Fatal {
                message: "Asset error".into(),
                source: Some(source.into()),
            },
        }
    }
}

impl From<LegalVoteError> for SignalingModuleError {
    fn from(source: LegalVoteError) -> Self {
        Self::CustomError {
            message: "Legal vote error".into(),
            source: Some(source.into()),
        }
    }
}

impl From<LegalVoteError> for ObjectStorageError {
    fn from(source: LegalVoteError) -> Self {
        Self::Other {
            message: "Legal vote error".into(),
            source: Some(source.into()),
        }
    }
}
