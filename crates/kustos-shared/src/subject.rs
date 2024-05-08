// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{fmt::Display, str::FromStr};

use snafu::Snafu;
use uuid::Uuid;

/// The error type is returned when parsing invalid values from strings.
///
/// Derived using [snafu::Snafu]
#[derive(Debug, Snafu)]
pub enum ParsingError {
    #[snafu(display("Invalid access method: `{method}`"))]
    InvalidAccessMethod { method: String },

    #[snafu(display("String was not a PolicyUser casbin string: `{user}`"))]
    PolicyUser { user: String },

    #[snafu(display("String was not a PolicyInvite casbin string: `{invite}`"))]
    PolicyInvite { invite: String },

    #[snafu(display("String was not a PolicyInternalGroup casbin string: `{group}"))]
    PolicyInternalGroup { group: String },

    #[snafu(display("String was not a PolicyOPGroup casbin string: `{group}`"))]
    PolicyOPGroup { group: String },

    #[snafu(display("Invalid UUID: {source}"), context(false))]
    Uuid {
        #[snafu(source(from(uuid::Error, Box::new)))]
        source: Box<uuid::Error>,
    },

    #[snafu(display("Custom: {message}"), whatever)]
    Custom {
        message: String,

        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, Some)))]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

/// Trait to tag a type as a subject
///
/// Types tagged with this trait need to implement the underlying internal conversion types as well.
/// The internal implementations take care of that for the subjects that are part of this API.
pub trait IsSubject {}

/// A uuid backed user identifier.
///
/// This crates requires your users to be identifiable by a uuid.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct PolicyUser(pub(crate) uuid::Uuid);

impl IsSubject for PolicyUser {}

impl PolicyUser {
    /// Create a ZERO policy user, e.g. for testing purposes
    pub const fn nil() -> Self {
        Self(Uuid::nil())
    }

    /// Create a policy user from a number, e.g. for testing purposes
    pub const fn from_u128(id: u128) -> Self {
        Self(Uuid::from_u128(id))
    }

    /// Generate a new random policy user
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Display for PolicyUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<uuid::Uuid> for PolicyUser {
    fn from(user: uuid::Uuid) -> Self {
        PolicyUser(user)
    }
}

impl FromStr for PolicyUser {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("user::") {
            Ok(PolicyUser(uuid::Uuid::from_str(
                s.trim_start_matches("user::"),
            )?))
        } else {
            PolicyUserSnafu { user: s.to_owned() }.fail()
        }
    }
}

impl AsRef<uuid::Uuid> for PolicyUser {
    fn as_ref(&self) -> &uuid::Uuid {
        &self.0
    }
}

/// A uuid backed invite identifier.
///
/// This crates requires the invites codes to be identifiable by a uuid.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct PolicyInvite(pub(crate) uuid::Uuid);

impl IsSubject for PolicyInvite {}

impl PolicyInvite {
    /// Create a ZERO policy invite, e.g. for testing purposes
    pub const fn nil() -> Self {
        Self(Uuid::nil())
    }

    /// Create a policy invite from a number, e.g. for testing purposes
    pub const fn from_u128(id: u128) -> Self {
        Self(Uuid::from_u128(id))
    }

    /// Generate a new random policy invite
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}

impl From<uuid::Uuid> for PolicyInvite {
    fn from(invite: uuid::Uuid) -> Self {
        PolicyInvite(invite)
    }
}

impl FromStr for PolicyInvite {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("invite::") {
            Ok(PolicyInvite(uuid::Uuid::from_str(
                s.trim_start_matches("invite::"),
            )?))
        } else {
            PolicyInviteSnafu {
                invite: s.to_owned(),
            }
            .fail()
        }
    }
}

impl AsRef<uuid::Uuid> for PolicyInvite {
    fn as_ref(&self) -> &uuid::Uuid {
        &self.0
    }
}

/// An internal group e.g. administrator, moderator, etc.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyRole(pub(crate) String);

impl IsSubject for PolicyRole {}

impl From<String> for PolicyRole {
    fn from(group: String) -> Self {
        PolicyRole(group)
    }
}

impl From<&str> for PolicyRole {
    fn from(group: &str) -> Self {
        PolicyRole(group.to_string())
    }
}

impl FromStr for PolicyRole {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("role::") {
            Ok(PolicyRole(s.trim_start_matches("role::").to_string()))
        } else {
            PolicyInternalGroupSnafu {
                group: s.to_owned(),
            }
            .fail()
        }
    }
}

impl AsRef<str> for PolicyRole {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

/// A user defined group, such as information from keycloak or LDAP
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyGroup(pub(crate) String);

impl IsSubject for PolicyGroup {}

impl From<String> for PolicyGroup {
    fn from(group: String) -> Self {
        PolicyGroup(group)
    }
}
impl From<&str> for PolicyGroup {
    fn from(group: &str) -> Self {
        PolicyGroup(group.to_string())
    }
}

impl FromStr for PolicyGroup {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("group::") {
            Ok(PolicyGroup(s.trim_start_matches("group::").to_string()))
        } else {
            PolicyOPGroupSnafu {
                group: s.to_owned(),
            }
            .fail()
        }
    }
}

/// Maps a PolicyUser to a PolicyRole
pub struct UserToRole(pub PolicyUser, pub PolicyRole);

/// Maps a PolicyUser to a PolicyGroup
pub struct UserToGroup(pub PolicyUser, pub PolicyGroup);

/// Maps a PolicyGroup to a PolicyRole
pub struct GroupToRole(pub PolicyGroup, pub PolicyRole);

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::subject::{ParsingError, PolicyInvite, PolicyUser};

    #[test]
    fn test_policy_invite_invalid_uuid() {
        let raw_invite = "invite::00000000-0000-0000c0000-000000000000";
        let parsing_result = PolicyInvite::from_str(raw_invite);
        assert!(
            matches!(parsing_result, Err(ParsingError::Uuid { source: _ }),),
            "Expected Uuid error, Got: {:?}",
            parsing_result,
        );
    }

    #[test]
    fn test_policy_user_invalid_uuid() {
        let raw_invite = "user::00000000-0000-0000c0000-000000000000";
        let parsing_result = PolicyUser::from_str(raw_invite);
        assert!(
            matches!(parsing_result, Err(ParsingError::Uuid { source: _ }),),
            "Expected Uuid error, Got: {:?}",
            parsing_result,
        );
    }
}
