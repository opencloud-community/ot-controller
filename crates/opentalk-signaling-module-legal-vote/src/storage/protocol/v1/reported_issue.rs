// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_types_common::users::UserId;
use opentalk_types_signaling_legal_vote::issue::Issue;

use crate::storage::v1::UserInfo;

/// Represents an issue reported during the vote by a participant.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReportedIssue {
    /// User information of the participant that reported the issue.
    ///
    /// `None` if the vote is hidden.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub user_info: Option<UserInfo>,

    /// The kind of issue that the user encountered.
    #[serde(flatten)]
    pub issue: Issue,
}

impl ReportedIssue {
    /// Retrieves the user IDs referenced in the reported issue.
    ///
    /// Returns a set containing the user ID if the issue has associated user information.
    pub fn get_referenced_user_ids(&self) -> BTreeSet<UserId> {
        self.user_info.iter().map(|info| info.issuer).collect()
    }
}

#[cfg(test)]
mod serde_tests {
    use opentalk_types_signaling::ParticipantId;
    use opentalk_types_signaling_legal_vote::issue::{TechnicalIssue, TechnicalIssueKind};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialization() {
        let produced = serde_json::to_value(ReportedIssue {
            user_info: Some(UserInfo {
                issuer: UserId::from_u128(1),
                participant_id: ParticipantId::from_u128(2),
            }),
            issue: Issue::Technical(TechnicalIssue {
                kind: TechnicalIssueKind::Audio,
                description: None,
            }),
        })
        .unwrap();

        let expected = json!({
            "issuer": "00000000-0000-0000-0000-000000000001",
            "participant_id": "00000000-0000-0000-0000-000000000002",
            "kind": "audio",
        });

        assert_eq!(produced, expected);

        let produced = serde_json::to_value(ReportedIssue {
            user_info: None,
            issue: Issue::Technical(TechnicalIssue {
                kind: TechnicalIssueKind::Audio,
                description: None,
            }),
        })
        .unwrap();

        let expected = json!({
            "kind": "audio",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization() {
        let produced: ReportedIssue = serde_json::from_value(json!({
            "issuer": "00000000-0000-0000-0000-000000000001",
            "participant_id": "00000000-0000-0000-0000-000000000002",
            "kind": "audio",
        }))
        .unwrap();

        let expected = ReportedIssue {
            user_info: Some(UserInfo {
                issuer: UserId::from_u128(1),
                participant_id: ParticipantId::from_u128(2),
            }),
            issue: Issue::Technical(TechnicalIssue {
                kind: TechnicalIssueKind::Audio,
                description: None,
            }),
        };

        assert_eq!(produced, expected);

        let produced: ReportedIssue = serde_json::from_value(json!({
            "issue": "technical",
            "kind": "audio",
        }))
        .unwrap();

        let expected = ReportedIssue {
            user_info: None,
            issue: Issue::Technical(TechnicalIssue {
                kind: TechnicalIssueKind::Audio,
                description: None,
            }),
        };

        assert_eq!(produced, expected);
    }
}
