// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

/// The quota types that can be enforced on tenants.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(
    feature = "clap",
    derive(clap::ValueEnum),
    clap(rename_all = "snake_case")
)]
pub enum QuotaType {
    /// This quota limits the total amount of data, measured bytes, that can be
    /// stored by the tenant. This is a soft limit which allows the user to store
    /// files as long as their usage is below the limit. Once the limit is reached
    /// or exceeded, no new data can be stored.
    MaxStorage,

    /// This quota restricts the total duration for which a tenant can utilize a
    /// meeting room, measured in seconds.
    RoomTimeLimitSecs,

    /// This quota sets a limit on the number of participants that can join a room.
    RoomParticipantLimit,

    /// Generic quota type.
    #[cfg_attr(feature = "serde", serde(untagged))]
    #[cfg_attr(feature = "clap", clap(skip))]
    #[strum(default)]
    Other(String),
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

    #[cfg(feature = "serde")]
    #[test]
    fn quota_type_json() {
        use std::collections::BTreeMap;

        use serde_json::json;
        let quota = BTreeMap::from([
            (QuotaType::MaxStorage, 11u64),
            (QuotaType::RoomTimeLimitSecs, 12u64),
            (QuotaType::RoomParticipantLimit, 13u64),
            (QuotaType::Other("this_is_somethingElse".to_string()), 14u64),
        ]);
        let quota_json_repr =
            serde_json::to_value(quota.clone()).expect("QuotaType must be serializable");

        assert_eq!(
            quota_json_repr,
            json!({
                "max_storage": 11,
                "room_time_limit_secs": 12,
                "room_participant_limit": 13,
                "this_is_somethingElse": 14
            })
        );
        assert_eq!(
            quota,
            serde_json::from_value(quota_json_repr).expect("Must be deserialize")
        );
    }

    #[cfg(feature = "clap")]
    #[test]
    fn quota_type_string() {
        use std::str::FromStr;

        assert_eq!(
            QuotaType::from_str("max_storage").unwrap(),
            QuotaType::MaxStorage
        );
        assert_eq!(
            QuotaType::from_str("room_time_limit_secs").unwrap(),
            QuotaType::RoomTimeLimitSecs
        );
        assert_eq!(
            QuotaType::from_str("room_participant_limit").unwrap(),
            QuotaType::RoomParticipantLimit
        );
        assert_eq!(
            QuotaType::from_str("this_is_somethingElse").unwrap(),
            QuotaType::Other("this_is_somethingElse".to_string())
        );
    }
}
