// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

/// A trait for defining data for peers sent to the frontend of a signaling module.
pub trait SignalingModulePeerFrontendData: Serialize + DeserializeOwned + std::fmt::Debug {
    /// The namespace which is used to tag the signaling module participant data
    const NAMESPACE: Option<&'static str>;
}

impl SignalingModulePeerFrontendData for () {
    const NAMESPACE: Option<&'static str> = None;
}
