// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains functions that are used in different areas of the OpenTalk

#[allow(unused_imports)]
use crate::imports::*;

/// Helper function to deserialize Option<Option<T>>
/// https://github.com/serde-rs/serde/issues/984
#[cfg(feature = "serde")]
pub(super) fn deserialize_some<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}

/// Validates a recurrence pattern for an event
#[cfg(feature = "serde")]
pub fn validate_recurrence_pattern(pattern: &[String]) -> Result<(), ValidationError> {
    if pattern.len() > 4 {
        return Err(ValidationError::new("too_many_recurrence_patterns"));
    }

    if pattern.iter().any(|p| p.len() > 1024) {
        return Err(ValidationError::new("recurrence_pattern_too_large"));
    }

    Ok(())
}
