// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains functions that are used in different areas of the OpenTalk

#![cfg(feature = "serde")]

#[allow(unused_imports)]
use crate::imports::*;

/// Helper function to deserialize Option<Option<T>>
/// https://github.com/serde-rs/serde/issues/984
pub(super) fn deserialize_some<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Test {
        #[serde(default, deserialize_with = "deserialize_some")]
        test: Option<Option<String>>,
    }

    #[test]
    fn deserialize_option_option() {
        let none = "{}";
        let some_none = r#"{"test":null}"#;
        let some_some = r#"{"test":"test"}"#;

        assert_eq!(
            serde_json::from_str::<Test>(none).unwrap(),
            Test { test: None }
        );
        assert_eq!(
            serde_json::from_str::<Test>(some_none).unwrap(),
            Test { test: Some(None) }
        );
        assert_eq!(
            serde_json::from_str::<Test>(some_some).unwrap(),
            Test {
                test: Some(Some("test".into()))
            }
        );
    }
}
