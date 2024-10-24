// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::state::ModeratorFrontendData;

/// The state of the `moderation` module.
///
/// This struct is sent to the participant in the `join_success` message
/// when they join successfully to the meeting.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModerationState {
    /// Moderation module data that is only avaialble for moderators
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub moderator_data: Option<ModeratorFrontendData>,

    /// Is raise hands enabled
    pub raise_hands_enabled: bool,
}

#[cfg(feature = "serde")]
impl opentalk_types_signaling::SignalingModuleFrontendData for ModerationState {
    const NAMESPACE: Option<&'static str> = Some(crate::NAMESPACE);
}
