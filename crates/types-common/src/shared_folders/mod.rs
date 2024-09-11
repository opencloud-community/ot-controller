// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Data types for handling shared folders.

/// The namespace string for the signaling module
pub const NAMESPACE: &str = "shared_folder";

mod shared_folder;
mod shared_folder_access;

pub use shared_folder::SharedFolder;
pub use shared_folder_access::SharedFolderAccess;
