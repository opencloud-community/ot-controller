// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Error from the `meeting_report` module namespace
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "error", rename_all = "snake_case")
)]
pub enum Error {
    /// The requesting user has insufficient permissions for the operation
    InsufficientPermissions,
    /// The requesting user has exceeded their storage
    StorageExceeded,
    /// Internal error while generating the report
    Generate,
    /// Internal error while saving the report
    Storage,
}
