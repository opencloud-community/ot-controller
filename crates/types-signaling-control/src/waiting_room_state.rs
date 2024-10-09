// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// The namespace string for the waiting room state
pub const NAMESPACE: &str = "waiting_room_state";

/// The waiting room state of a meeting participant
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum WaitingRoomState {
    /// The participant currently is in the waiting room
    Waiting,

    /// The participant has been accepted into the meeting
    Accepted,
}

#[cfg(feature = "serde")]
impl opentalk_types_signaling::SignalingModulePeerFrontendData for WaitingRoomState {
    const NAMESPACE: Option<&'static str> = Some(NAMESPACE);
}
