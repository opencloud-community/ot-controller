// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Common types related to the shared_folder module

use crate::signaling::Role;

#[allow(unused_imports)]
use crate::imports::*;

/// Information required to access a shared folder
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SharedFolderAccess {
    /// Shared folder URL
    pub url: String,

    /// Password required to access the shared folder
    pub password: String,
}

/// Information about a shared folder containing
/// read and optional write access
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "redis",
    derive(ToRedisArgs, FromRedisValue),
    to_redis_args(serde),
    from_redis_value(serde)
)]
pub struct SharedFolder {
    /// Read access information for the shared folder
    pub read: SharedFolderAccess,

    /// Read-write access information for the shared folder
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub read_write: Option<SharedFolderAccess>,
}

#[cfg(feature = "serde")]
impl SignalingModuleFrontendData for SharedFolder {
    const NAMESPACE: Option<&'static str> = Some(crate::signaling::shared_folder::NAMESPACE);
}

impl SharedFolder {
    /// Get an equivalent shared folder, cut down to match the signaling role
    pub fn for_signaling_role(self, role: Role) -> Self {
        if role.is_moderator() {
            self
        } else {
            self.without_write_access()
        }
    }

    /// Get an equivalent shared folder, with write access removed
    pub fn without_write_access(self) -> Self {
        Self {
            read_write: None,
            ..self
        }
    }

    /// Get an equivalent shared folder, with write access added or replaced
    pub fn with_write_access(self, write_access: SharedFolderAccess) -> Self {
        Self {
            read_write: Some(write_access),
            ..self
        }
    }
}
