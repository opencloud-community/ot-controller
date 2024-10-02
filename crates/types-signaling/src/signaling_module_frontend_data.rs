// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// A trait for defining data sent to the frontend of a signaling module.
pub trait SignalingModuleFrontendData:
    serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug
{
    /// The namespace which is used to tag the signaling module data
    const NAMESPACE: Option<&'static str>;
}

impl SignalingModuleFrontendData for () {
    const NAMESPACE: Option<&'static str> = None;
}

impl SignalingModuleFrontendData for opentalk_types_common::shared_folders::SharedFolder {
    const NAMESPACE: Option<&'static str> = Some(opentalk_types_common::shared_folders::NAMESPACE);
}
