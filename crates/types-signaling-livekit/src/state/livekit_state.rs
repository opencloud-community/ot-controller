// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::Credentials;

/// Signaling event to pass information about the livekit server around
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LiveKitState(pub Credentials);

#[cfg(feature = "serde")]
impl opentalk_types_signaling::SignalingModuleFrontendData for LiveKitState {
    const NAMESPACE: Option<&'static str> = Some(crate::NAMESPACE);
}
