// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Frontend data for `media` namespace

use std::collections::BTreeSet;

use super::ParticipantSpeakingState;
use crate::core::ParticipantId;
#[allow(unused_imports)]
use crate::imports::*;

/// The state of the `media` module.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MediaState {
    /// Whether the participant has permission to share the screen
    pub is_presenter: bool,

    /// The list of recent or currently active speakers in the conference
    pub speakers: Vec<ParticipantSpeakingState>,

    /// Force mute state is enabled
    pub force_mute: ForceMuteState,
}

#[cfg(feature = "serde")]
impl SignalingModuleFrontendData for MediaState {
    const NAMESPACE: Option<&'static str> = Some(super::NAMESPACE);
}

/// Moderation module state that is visible only to moderators
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "type", rename_all = "snake_case")
)]
pub enum ForceMuteState {
    /// The force mute state is disabled, participants are allowed to unmute
    #[default]
    Disabled,
    /// The force mute state is enabled, only the participants part of `allow_list` are allowed to unmute
    Enabled {
        /// The list of participants that are still allowed to unmute
        allow_list: BTreeSet<ParticipantId>,
    },
}

impl ForceMuteState {
    /// Set the force mute state to enabled with an empty allow list.
    pub fn set_enabled(&mut self) {
        if self == &Self::Disabled {
            *self = Self::Enabled {
                allow_list: BTreeSet::new(),
            };
        }
    }

    /// Set the force mute state to disabled.
    pub fn set_disabled(&mut self) {
        *self = Self::Disabled;
    }

    /// Returns `true` if the force mute state is [`Enabled`].
    ///
    /// [`Enabled`]: ForceMuteState::Enabled
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        matches!(self, Self::Enabled { .. })
    }
}

#[cfg(test)]
mod tests {

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_force_mute_enabled() {
        use std::collections::BTreeSet;

        use serde_json::json;

        use super::ForceMuteState;

        let serialized = json!({
            "type": "enabled",
            "allow_list": [
                "00000000-0000-0000-0000-000000000001",
                "00000000-0000-0000-0000-000000000002"
            ]
        });
        let force_mute = ForceMuteState::Enabled {
            allow_list: BTreeSet::from([1u128.into(), 2u128.into()]),
        };

        assert_eq!(
            serialized,
            serde_json::to_value(force_mute).expect("Must be serializable")
        )
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_force_mute_disabled() {
        use serde_json::json;

        use super::ForceMuteState;

        let serialized = json!({
            "type": "disabled",
        });
        let force_mute = ForceMuteState::Disabled;

        assert_eq!(
            serialized,
            serde_json::to_value(force_mute).expect("Must be serializable")
        )
    }
}
