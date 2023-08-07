// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use uuid::Uuid;

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

impl From<uuid::Uuid> for PolicyUser {
    fn from(user: uuid::Uuid) -> Self {
        PolicyUser(user)
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

/// Maps a PolicyUser to a PolicyRole
pub struct UserToRole(pub PolicyUser, pub PolicyRole);

/// Maps a PolicyUser to a PolicyGroup
pub struct UserToGroup(pub PolicyUser, pub PolicyGroup);

/// Maps a PolicyGroup to a PolicyRole
pub struct GroupToRole(pub PolicyGroup, pub PolicyRole);
