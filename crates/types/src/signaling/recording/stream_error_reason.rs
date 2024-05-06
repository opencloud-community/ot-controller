// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

/// An error signal from the Recorder
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
pub struct StreamErrorReason {
    /// The error that happened
    pub code: String,
    /// The reason as to why that error happened
    pub message: String,
}
