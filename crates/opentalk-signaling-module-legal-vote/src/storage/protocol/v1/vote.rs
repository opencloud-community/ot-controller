// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_types_common::users::UserId;
use opentalk_types_signaling_legal_vote::{token::Token, vote::VoteOption};

use crate::storage::v1::UserInfo;

/// A vote entry mapped to a specific user.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Vote {
    /// User information of the voting participant.
    ///
    /// `None` if the vote is hidden.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub user_info: Option<UserInfo>,

    /// The token used for voting.
    pub token: Token,

    /// The chosen vote option.
    pub option: VoteOption,
}

impl Vote {
    /// Retrieves the user IDs referenced in the vote.
    ///
    /// Returns a set of user IDs if the vote has associated user information.
    pub fn get_referenced_user_ids(&self) -> BTreeSet<UserId> {
        self.user_info.iter().map(|info| info.issuer).collect()
    }
}

#[cfg(test)]
mod serde_tests {
    use std::str::FromStr;

    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialization() {
        let produced = serde_json::to_value(Vote {
            user_info: Some(UserInfo {
                issuer: UserId::from_u128(1),
                participant_id: ParticipantId::from_u128(2),
            }),
            token: Token::from_str("1111Cn8eVZg").unwrap(),
            option: VoteOption::Yes,
        })
        .unwrap();

        let expected = json!({
            "issuer": "00000000-0000-0000-0000-000000000001",
            "participant_id": "00000000-0000-0000-0000-000000000002",
            "token": "1111Cn8eVZg",
            "option": "yes",
        });

        assert_eq!(produced, expected);

        let produced = serde_json::to_value(Vote {
            user_info: None,
            token: Token::from_str("1111Cn8eVZg").unwrap(),
            option: VoteOption::Yes,
        })
        .unwrap();

        let expected = json!({
            "token": "1111Cn8eVZg",
            "option": "yes",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization() {
        let produced: Vote = serde_json::from_value(json!({
            "issuer": "00000000-0000-0000-0000-000000000001",
            "participant_id": "00000000-0000-0000-0000-000000000002",
            "token": "1111Cn8eVZg",
            "option": "yes",
        }))
        .unwrap();

        let expected = Vote {
            user_info: Some(UserInfo {
                issuer: UserId::from_u128(1),
                participant_id: ParticipantId::from_u128(2),
            }),
            token: Token::from_str("1111Cn8eVZg").unwrap(),
            option: VoteOption::Yes,
        };

        assert_eq!(produced, expected);

        let produced: Vote = serde_json::from_value(json!({
            "token": "1111Cn8eVZg",
            "option": "yes",
        }))
        .unwrap();

        let expected = Vote {
            user_info: None,
            token: Token::from_str("1111Cn8eVZg").unwrap(),
            option: VoteOption::Yes,
        };

        assert_eq!(produced, expected);
    }
}
