// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling protocol for the `legal-vote` namespace.

pub mod v1;

mod new_protocol;

pub use new_protocol::NewProtocol;

/// Represents a protocol with a version and protocol entries.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Protocol {
    /// The version of the protocol.
    pub version: u8,

    /// The protocol entries, stored as a raw JSON value.
    pub entries: Box<serde_json::value::RawValue>,
}

#[cfg(test)]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::{json, value::RawValue};

    use super::*;

    #[test]
    fn serialization() {
        let produced = serde_json::to_value(Protocol {
            version: 1,
            entries: *Box::new(
                RawValue::from_string("{ \"test\": \"test\" }".to_string()).unwrap(),
            ),
        })
        .unwrap();

        let expected = json!({
            "version": 1,
            "entries": {
                "test": "test",
            }
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization() {
        let produced: Result<Protocol, _> = serde_json::from_value(json!({
            "version": 1,
            "entries": "Test"
        }));

        assert!(produced.is_ok());
    }
}
