// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_types_common::users::UserId;
use opentalk_types_signaling_legal_vote::vote::StopKind as TypesStopKind;

/// Represents the different reasons a vote can be stopped.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopKind {
    /// A normal vote stop issued by a user, containing the `UserId` of the issuer.
    ByUser(UserId),

    /// The vote was stopped automatically because all allowed users have voted.
    Auto,

    /// The vote expired after reaching the set duration.
    Expired,
}

impl From<StopKind> for TypesStopKind {
    fn from(value: StopKind) -> Self {
        match value {
            StopKind::ByUser(user_id) => Self::ByUser {
                stopped_by: user_id,
            },
            StopKind::Auto => Self::Auto,
            StopKind::Expired => Self::Expired,
        }
    }
}

impl StopKind {
    /// Retrieves the user IDs referenced in the stop event.
    ///
    /// Returns a set containing the user ID if the stop was issued by a user, or an empty set otherwise.
    pub fn get_referenced_user_ids(&self) -> BTreeSet<UserId> {
        match self {
            StopKind::ByUser(user_id) => BTreeSet::from_iter([*user_id]),
            StopKind::Auto | StopKind::Expired => BTreeSet::new(),
        }
    }
}

#[cfg(test)]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialization_by_user_stop_kind() {
        let produced = serde_json::to_value(StopKind::ByUser(UserId::from_u128(1))).unwrap();

        let expected = json!({
            "by_user": "00000000-0000-0000-0000-000000000001",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization_by_user_stop_kind() {
        let produced: StopKind = serde_json::from_value(json!({
            "by_user": "00000000-0000-0000-0000-000000000001",
        }))
        .unwrap();

        let expected = StopKind::ByUser(UserId::from_u128(1));

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialization_auto_stop_kind() {
        let produced = serde_json::to_value(StopKind::Auto).unwrap();

        let expected = json!("auto");

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization_auto_stop_kind() {
        let produced: StopKind = serde_json::from_value(json!("auto")).unwrap();

        let expected = StopKind::Auto;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialization_expired_stop_kind() {
        let produced = serde_json::to_value(StopKind::Expired).unwrap();

        let expected = json!("expired");

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization_expired_stop_kind() {
        let produced: StopKind = serde_json::from_value(json!("expired")).unwrap();

        let expected = StopKind::Expired;

        assert_eq!(produced, expected);
    }
}
