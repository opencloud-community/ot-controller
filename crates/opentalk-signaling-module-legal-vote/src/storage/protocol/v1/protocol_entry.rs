// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use opentalk_types_common::users::UserId;

use crate::storage::protocol::v1::VoteEvent;

/// A legal vote protocol entry, containing an event and metadata.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    redis_args::ToRedisArgs,
    redis_args::FromRedisValue,
    serde::Serialize,
    serde::Deserialize,
)]
#[to_redis_args(serde)]
#[from_redis_value(serde)]
pub struct ProtocolEntry {
    /// The time when the entry was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,

    /// The event associated with this entry.
    pub event: VoteEvent,
}

impl ProtocolEntry {
    /// Creates a new protocol entry with the current timestamp.
    pub fn new(event: VoteEvent) -> Self {
        Self::new_with_time(Utc::now(), event)
    }

    /// Creates a new protocol entry with an optional timestamp.
    pub fn new_with_optional_time(timestamp: Option<DateTime<Utc>>, event: VoteEvent) -> Self {
        Self { timestamp, event }
    }

    /// Creates a new protocol entry using the provided `timestamp`.
    pub fn new_with_time(timestamp: DateTime<Utc>, event: VoteEvent) -> Self {
        Self::new_with_optional_time(Some(timestamp), event)
    }

    /// Retrieves the user IDs referenced in the protocol entry's event.
    pub fn get_referenced_user_ids(&self) -> BTreeSet<UserId> {
        self.event.get_referenced_user_ids()
    }
}

#[cfg(test)]
mod serde_tests {
    use chrono::TimeZone;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::storage::v1::MaybeUserInfo;

    #[test]
    fn serialization() {
        let produced = serde_json::to_value(ProtocolEntry {
            timestamp: Some(Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap()),
            event: VoteEvent::UserJoined(MaybeUserInfo { inner: None }),
        })
        .unwrap();

        let expected = json!({
            "event": {
                "event": "user_joined",
            },
            "timestamp":"2025-01-01T00:00:00Z",
        });

        assert_eq!(produced, expected);

        let produced = serde_json::to_value(ProtocolEntry {
            timestamp: None,
            event: VoteEvent::UserJoined(MaybeUserInfo { inner: None }),
        })
        .unwrap();

        let expected = json!({
            "event": {
                "event": "user_joined",
            }
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization() {
        let produced: ProtocolEntry = serde_json::from_value(json!({
        "event": {
            "event": "user_joined",
        },
            "timestamp":"2025-01-01T00:00:00Z",
        }))
        .unwrap();

        let expected = ProtocolEntry {
            timestamp: Some(Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap()),
            event: VoteEvent::UserJoined(MaybeUserInfo { inner: None }),
        };

        assert_eq!(produced, expected);

        let produced: ProtocolEntry = serde_json::from_value(json!({
            "event": {
                "event": "user_joined",
            }
        }))
        .unwrap();

        let expected = ProtocolEntry {
            timestamp: None,
            event: VoteEvent::UserJoined(MaybeUserInfo { inner: None }),
        };

        assert_eq!(produced, expected);
    }
}
