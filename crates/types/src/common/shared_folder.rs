// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Common types related to the shared_folder module

#[allow(unused_imports)]
use crate::imports::*;
use crate::{signaling::Role, utils::ExampleData};

/// Information required to access a shared folder
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(SharedFolderAccess::example_data())),
)]
pub struct SharedFolderAccess {
    /// Shared folder URL
    pub url: String,

    /// Password required to access the shared folder
    pub password: String,
}

impl ExampleData for SharedFolderAccess {
    fn example_data() -> Self {
        Self {
            url: "https://cloud.example.com/shares/abc123".to_string(),
            password: "v3rys3cr3t".to_string(),
        }
    }
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
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(SharedFolder::example_data())),
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

impl ExampleData for SharedFolder {
    fn example_data() -> Self {
        Self {
            read: SharedFolderAccess::example_data(),
            read_write: None,
        }
    }
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
