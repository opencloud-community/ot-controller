// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

/// A trait for defining data sent to the frontend of a signaling module.
pub trait SignalingModuleFrontendData: Serialize + DeserializeOwned + std::fmt::Debug {
    /// The namespace which is used to tag the signaling module data
    const NAMESPACE: Option<&'static str>;
}

impl SignalingModuleFrontendData for () {
    const NAMESPACE: Option<&'static str> = None;
}
