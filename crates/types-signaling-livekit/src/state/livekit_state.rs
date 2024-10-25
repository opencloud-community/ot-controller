// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{Credentials, MicrophoneRestrictionState};

/// Signaling event to pass information about the livekit server around
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LiveKitState {
    /// The current credentials of the livekit instance
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub credentials: Credentials,

    /// The current state of microphone restrictions
    pub microphone_restriction_state: MicrophoneRestrictionState,
}

#[cfg(feature = "serde")]
impl opentalk_types_signaling::SignalingModuleFrontendData for LiveKitState {
    const NAMESPACE: Option<&'static str> = Some(crate::NAMESPACE);
}
