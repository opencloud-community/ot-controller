// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_types_common::users::UserId;
use opentalk_types_signaling_legal_vote::cancel::{Cancel as TypesCancel, CancelReason};

/// Represents a vote cancellation, including the issuer and reason.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Cancel {
    /// The user ID of the issuer of the cancellation.
    pub issuer: UserId,

    /// The reason for the cancellation.
    #[serde(flatten)]
    pub reason: CancelReason,
}

impl Cancel {
    /// Retrieves the user IDs referenced in the cancellation event.
    ///
    /// Returns a set containing the user ID of the issuer.
    pub fn get_referenced_user_ids(&self) -> BTreeSet<UserId> {
        BTreeSet::from_iter([self.issuer])
    }
}

impl From<Cancel> for TypesCancel {
    fn from(value: Cancel) -> Self {
        Self {
            issuer: value.issuer,
            reason: value.reason,
        }
    }
}

#[cfg(test)]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialization() {
        let produced = serde_json::to_value(Cancel {
            issuer: UserId::from_u128(1),
            reason: CancelReason::RoomDestroyed,
        })
        .unwrap();

        let expected = json!({
            "issuer": "00000000-0000-0000-0000-000000000001",
            "reason": "room_destroyed",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization() {
        let produced: Cancel = serde_json::from_value(json!({
            "issuer": "00000000-0000-0000-0000-000000000001",
            "reason": "room_destroyed",
        }))
        .unwrap();

        let expected = Cancel {
            issuer: UserId::from_u128(1),
            reason: CancelReason::RoomDestroyed,
        };

        assert_eq!(produced, expected);
    }
}
