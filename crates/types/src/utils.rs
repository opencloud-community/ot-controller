// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

#[cfg(feature = "serde")]
pub mod duration_seconds_option {
    use super::*;
    use std::time::Duration;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds: Option<u64> = Deserialize::deserialize(deserializer)?;
        Ok(seconds.map(Duration::from_secs))
    }

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

#[cfg(feature = "serde")]
pub mod duration_seconds {
    use super::*;
    use std::time::Duration;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds: u64 = Deserialize::deserialize(deserializer)?;
        Ok(Duration::from_secs(seconds))
    }

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }
}
