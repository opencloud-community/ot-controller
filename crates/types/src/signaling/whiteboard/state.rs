// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Frontend data for `whiteboard` namespace

#[allow(unused_imports)]
use crate::imports::*;

use url::Url;

/// The state of the `whiteboard` module.
///
/// This struct is sent to the participant in the `join_success` message
/// when they join successfully to the meeting.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case", tag = "status", content = "url")
)]
pub enum WhiteboardState {
    /// Whiteboard is not initialized
    NotInitialized,

    /// Whiteboard is initializing
    Initializing,

    /// Whiteboard is initialized
    Initialized(Url),
}

#[cfg(feature = "serde")]
impl SignalingModuleFrontendData for WhiteboardState {
    const NAMESPACE: Option<&'static str> = Some(super::NAMESPACE);
}
