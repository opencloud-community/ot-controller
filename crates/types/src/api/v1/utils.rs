// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains functions that are used in different areas of the OpenTalk

#![cfg(feature = "serde")]

use core::fmt;
use std::{marker::PhantomData, str::FromStr};

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

/// Validates a recurrence pattern for an event
pub fn validate_recurrence_pattern(pattern: &[String]) -> Result<(), ValidationError> {
    if pattern.len() > 4 {
        return Err(ValidationError::new("too_many_recurrence_patterns"));
    }

    if pattern.iter().any(|p| p.len() > 1024) {
        return Err(ValidationError::new("recurrence_pattern_too_large"));
    }

    Ok(())
}

/// Helper function to deserialize comma-separated values
pub fn comma_separated<'de, V, T, D>(deserializer: D) -> Result<V, D::Error>
where
    V: FromIterator<T>,
    T: FromStr,
    T::Err: fmt::Display,
    D: Deserializer<'de>,
{
    struct CommaSeparated<V, T>(PhantomData<(T, V)>);

    impl<'de, V, T> serde::de::Visitor<'de> for CommaSeparated<V, T>
    where
        V: FromIterator<T>,
        T: FromStr,
        T::Err: fmt::Display,
    {
        type Value = V;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string containing comma-separated elements")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let iter = s.split(',').map(FromStr::from_str);
            iter.collect::<Result<_, _>>().map_err(de::Error::custom)
        }
    }

    let visitor = CommaSeparated(PhantomData);
    deserializer.deserialize_str(visitor)
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

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
