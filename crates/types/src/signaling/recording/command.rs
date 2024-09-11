// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `recording` namespace

use std::collections::BTreeSet;

use opentalk_types_common::streaming::StreamingTargetId;

#[allow(unused_imports)]
use crate::imports::*;

/// Commands for the `recording` namespace
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "action", rename_all = "snake_case")
)]
pub enum RecordingCommand {
    /// Set the consent status for a specific recording
    SetConsent(SetConsent),

    /// Starts a stream
    StartStream(StartStreaming),

    /// Pauses a stream
    PauseStream(PauseStreaming),

    /// Stops a stream
    StopStream(StopStreaming),
}

impl From<StopStreaming> for RecordingCommand {
    fn from(value: StopStreaming) -> Self {
        Self::StopStream(value)
    }
}

/// Data for the `set_consent` recording command
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SetConsent {
    /// Flag indicating whether the participant consents to being recorded
    pub consent: bool,
}

impl From<SetConsent> for RecordingCommand {
    fn from(value: SetConsent) -> Self {
        Self::SetConsent(value)
    }
}

/// Data for the `start` streaming command
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StartStreaming {
    /// Id of the to be started stream
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeSet::is_empty")
    )]
    pub target_ids: BTreeSet<StreamingTargetId>,
}

/// Data for the `pause` streaming command
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PauseStreaming {
    /// Id of the to be paused stream
    pub target_ids: BTreeSet<StreamingTargetId>,
}

/// Data for the `stop` streaming command
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StopStreaming {
    /// Id of the to be stopped stream
    pub target_ids: BTreeSet<StreamingTargetId>,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use opentalk_types_common::streaming::StreamingTargetId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::RecordingCommand;

    #[test]
    fn simple_de_serialization_consent() {
        let json = json!({
            "action": "set_consent",
            "consent": true,
        });

        let value = RecordingCommand::SetConsent(super::SetConsent { consent: true });

        let serialized = serde_json::to_value(&value);

        assert!(serialized.is_ok());
        assert_eq!(json, serialized.unwrap(), "serialized JSON matches");

        let deserialized = serde_json::from_value(json);

        assert!(deserialized.is_ok());
        assert_eq!(value, deserialized.unwrap(), "deserialized JSON matches");
    }

    #[test]
    fn simple_de_serialization_start_stream() {
        let json = json!({
            "action": "start_stream",
            "target_ids": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-000000000001", "00000000-0000-0000-0000-000000000002"],
        });

        let value = RecordingCommand::StartStream(super::StartStreaming {
            target_ids: BTreeSet::from([
                StreamingTargetId::from_u128(0),
                StreamingTargetId::from_u128(1),
                StreamingTargetId::from_u128(2),
            ]),
        });

        let serialized = serde_json::to_value(&value);

        assert!(serialized.is_ok());
        assert_eq!(json, serialized.unwrap(), "serialized JSON matches");

        let deserialized = serde_json::from_value(json);

        assert!(deserialized.is_ok());
        assert_eq!(value, deserialized.unwrap(), "deserialized JSON matches");
    }

    #[test]
    fn simple_de_serialization_pause_stream() {
        let json = json!({
            "action": "pause_stream",
            "target_ids": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-000000000001", "00000000-0000-0000-0000-000000000002"],
        });

        let value = RecordingCommand::PauseStream(super::PauseStreaming {
            target_ids: BTreeSet::from([
                StreamingTargetId::from_u128(0),
                StreamingTargetId::from_u128(1),
                StreamingTargetId::from_u128(2),
            ]),
        });

        let serialized = serde_json::to_value(&value);

        assert!(serialized.is_ok());
        assert_eq!(json, serialized.unwrap(), "serialized JSON matches");

        let deserialized = serde_json::from_value(json);

        assert!(deserialized.is_ok());
        assert_eq!(value, deserialized.unwrap(), "deserialized JSON matches");
    }

    #[test]
    fn simple_de_serialization_stop_stream() {
        let json = json!({
            "action": "stop_stream",
            "target_ids": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-000000000001", "00000000-0000-0000-0000-000000000002"],
        });

        let value = RecordingCommand::StopStream(super::StopStreaming {
            target_ids: BTreeSet::from([
                StreamingTargetId::from_u128(0),
                StreamingTargetId::from_u128(1),
                StreamingTargetId::from_u128(2),
            ]),
        });

        let serialized = serde_json::to_value(&value);

        assert!(serialized.is_ok());
        assert_eq!(json, serialized.unwrap(), "serialized JSON matches");

        let deserialized = serde_json::from_value(json);

        assert!(deserialized.is_ok());
        assert_eq!(value, deserialized.unwrap(), "deserialized JSON matches");
    }
}
