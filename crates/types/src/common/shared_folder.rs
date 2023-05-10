// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Common types related to the shared_folder module

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
