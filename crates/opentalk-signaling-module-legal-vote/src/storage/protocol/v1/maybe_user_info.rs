// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_types_common::users::UserId;

use crate::storage::v1::UserInfo;

/// Wrapper type to satisfy Serde serialization/deserialization.
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MaybeUserInfo {
    /// User information of the participant.
    ///
    /// `None` if the vote is hidden.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub inner: Option<UserInfo>,
}

impl MaybeUserInfo {
    /// Retrieves the user IDs referenced in the `MaybeUserInfo`.
    ///
    /// Returns a set of user IDs if the `inner` contains user information.
    pub fn get_referenced_user_ids(&self) -> BTreeSet<UserId> {
        self.inner.iter().map(|info| info.issuer).collect()
    }
}

impl From<Option<UserInfo>> for MaybeUserInfo {
    fn from(value: Option<UserInfo>) -> Self {
        Self { inner: value }
    }
}

impl From<MaybeUserInfo> for Option<UserInfo> {
    fn from(value: MaybeUserInfo) -> Self {
        value.inner
    }
}

#[cfg(test)]
mod serde_tests {
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialization() {
        let produced = serde_json::to_value(MaybeUserInfo {
            inner: Some(UserInfo {
                issuer: UserId::from_u128(1),
                participant_id: ParticipantId::from_u128(2),
            }),
        })
        .unwrap();

        let expected = json!({
            "issuer": "00000000-0000-0000-0000-000000000001",
            "participant_id": "00000000-0000-0000-0000-000000000002",
        });

        assert_eq!(produced, expected);

        let produced = serde_json::to_value(MaybeUserInfo { inner: None }).unwrap();

        let expected = json!({});

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization() {
        let produced: MaybeUserInfo = serde_json::from_value(json!({
            "issuer": "00000000-0000-0000-0000-000000000001",
            "participant_id": "00000000-0000-0000-0000-000000000002",
        }))
        .unwrap();

        let expected = MaybeUserInfo {
            inner: Some(UserInfo {
                issuer: UserId::from_u128(1),
                participant_id: ParticipantId::from_u128(2),
            }),
        };

        assert_eq!(produced, expected);

        let produced: MaybeUserInfo = serde_json::from_value(json!({})).unwrap();

        let expected = MaybeUserInfo { inner: None };

        assert_eq!(produced, expected);
    }
}
