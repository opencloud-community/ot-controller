// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Module to use for (de-)serializing an [`Option<std::time::Duration>`] given in seconds.

use std::time::Duration;

use serde::{Deserialize, Deserializer, Serializer};

/// Deserialize function for the [`Option<Duration>`].
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    let seconds: Option<u64> = Deserialize::deserialize(deserializer)?;
    Ok(seconds.map(Duration::from_secs))
}

/// Serialize function for the [`Option<Duration>`].
pub fn serialize<S>(duration: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match duration {
        Some(duration) => serializer.serialize_u64(duration.as_secs()),
        None => serializer.serialize_none(),
    }
}
