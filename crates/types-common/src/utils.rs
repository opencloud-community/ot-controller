// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Helpful utilities used in this crate, but also useful outside of it.

#[allow(unused_imports)]
use crate::imports::*;

/// A trait for providing example data of an item.
pub trait ExampleData {
    /// Get an example instance of the current datatype.
    fn example_data() -> Self;
}

/// Module to use for (de-)serializing an [`Option<std::time::Duration>`] given in seconds.
#[cfg(feature = "serde")]
pub mod duration_seconds_option {
    use std::time::Duration;

    use super::*;

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
}

/// Module to use for (de-)serializing a [`std::time::Duration`] given in seconds.
#[cfg(feature = "serde")]
pub mod duration_seconds {
    use std::time::Duration;

    use super::*;

    /// Deserialize function for the [`Duration`].
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds: u64 = Deserialize::deserialize(deserializer)?;
        Ok(Duration::from_secs(seconds))
    }

    /// Serialize function for the [`Duration`].
    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }
}
