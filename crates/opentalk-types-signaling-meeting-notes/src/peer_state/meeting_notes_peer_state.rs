// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Peer frontend data for `meeting_notes` namespace

/// The state of other participants in the `meeting-notes` module.
///
/// This struct is sent to the participant in the `join_success` message
/// which will contain this information for each participant in the meeting.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub struct MeetingNotesPeerState {
    /// Read-only access
    pub readonly: bool,
}

#[cfg(feature = "serde")]
impl opentalk_types_signaling::SignalingModulePeerFrontendData for MeetingNotesPeerState {
    const NAMESPACE: Option<&'static str> = Some(crate::NAMESPACE);
}
