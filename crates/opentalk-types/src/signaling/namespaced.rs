// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{modules::ModuleId, time::Timestamp};

#[allow(unused_imports)]
use crate::imports::*;

/// An envelope of a command annotated with their respective module id.
///
/// This is used for WebSocket messages sent to the backend.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct NamespacedCommand<O> {
    /// The module to which the message is targeted
    #[cfg_attr(feature = "serde", serde(rename = "namespace"))]
    pub module: ModuleId,
    /// The payload of the message
    pub payload: O,
}

/// An envelope of an event annotated with their respective module id.
///
/// This is used for WebSocket messages sent to the frontend.
/// Similar to [`NamespacedCommand`], but includes a timestamp field.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct NamespacedEvent<O> {
    /// The namespace to which the message is targeted
    #[cfg_attr(feature = "serde", serde(rename = "namespace"))]
    pub module: ModuleId,
    /// The timestamp indicating the creation of the message
    pub timestamp: Timestamp,
    /// The payload of the message
    pub payload: O,
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use opentalk_types_common::utils::ExampleData;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_namespaced_command_serialization() {
        let module = ModuleId::example_data();
        let payload = "test_payload".to_string();

        let command = NamespacedCommand { module, payload };

        let expected = json!({
            "namespace": "mymodule",
            "payload": "test_payload"
        });

        let serialized = serde_json::to_value(&command).unwrap();
        assert_eq!(serialized, expected);

        let deserialized: NamespacedCommand<String> =
            serde_json::from_value(expected.clone()).unwrap();

        assert_eq!(deserialized, command);
    }

    #[test]
    fn test_namespaced_event_serialization() {
        let module = ModuleId::example_data();
        let timestamp = Timestamp::now();
        let payload = "test_payload".to_string();

        let event = NamespacedEvent {
            module,
            timestamp,
            payload,
        };

        let expected = json!({
            "namespace": "mymodule",
            "timestamp": timestamp,
            "payload": "test_payload"
        });

        let serialized = serde_json::to_value(&event).unwrap();
        assert_eq!(serialized, expected);

        let deserialized: NamespacedEvent<String> =
            serde_json::from_value(expected.clone()).unwrap();
        assert_eq!(deserialized, event);
    }
}
