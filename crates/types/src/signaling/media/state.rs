// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Frontend data for `media` namespace

#[allow(unused_imports)]
use crate::imports::*;

/// The state of the `media` module.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MediaState {
    /// Whether the participant has permission to share the screen
    pub is_presenter: bool,
}

#[cfg(feature = "serde")]
impl SignalingModuleFrontendData for MediaState {
    const NAMESPACE: Option<&'static str> = Some(super::NAMESPACE);
}
