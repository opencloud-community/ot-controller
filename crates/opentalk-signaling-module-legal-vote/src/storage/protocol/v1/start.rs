// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_types_common::users::UserId;
use opentalk_types_signaling_legal_vote::parameters::Parameters;

/// Represents the start of a vote, including the initiator and parameters.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Start {
    /// The user ID of the initiator.
    pub issuer: UserId,

    /// The parameters for the vote.
    pub parameters: Parameters,
}

impl Start {
    /// Retrieves the user IDs referenced in the start event.
    ///
    /// Returns a set of user IDs that includes the issuer and any users referenced in the vote parameters.
    pub fn get_referenced_user_ids(&self) -> BTreeSet<UserId> {
        let mut users = BTreeSet::from([self.issuer]);
        users.append(&mut self.parameters.get_referenced_user_ids());
        users
    }
}

#[cfg(test)]
mod serde_tests {
    use chrono::{TimeZone, Utc};
    use opentalk_types_signaling::ParticipantId;
    use opentalk_types_signaling_legal_vote::{
        user_parameters::{AllowedParticipants, Name, UserParameters},
        vote::{LegalVoteId, VoteKind},
    };
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialization() {
        let produced = serde_json::to_value(Start {
            issuer: UserId::from_u128(1),
            parameters: Parameters {
                initiator_id: ParticipantId::from_u128(1),
                legal_vote_id: LegalVoteId::from_u128(2),
                start_time: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
                max_votes: 5,
                allowed_users: None,
                inner: UserParameters {
                    kind: VoteKind::RollCall,
                    name: Name::try_from("Test Name").unwrap(),
                    subtitle: None,
                    topic: None,
                    allowed_participants: AllowedParticipants::try_from(vec![
                        ParticipantId::from_u128(3),
                    ])
                    .unwrap(),
                    enable_abstain: false,
                    auto_close: false,
                    duration: None,
                    create_pdf: false,
                    timezone: None,
                },
                token: None,
            },
        })
        .unwrap();

        let expected = json!({
            "issuer": "00000000-0000-0000-0000-000000000001",
            "parameters": {
                "initiator_id": "00000000-0000-0000-0000-000000000001",
                "legal_vote_id": "00000000-0000-0000-0000-000000000002",
                "start_time":"2025-01-01T00:00:00Z",
                "max_votes": 5,
                "kind": "roll_call",
                "name": "Test Name",
                "allowed_participants": [
                   "00000000-0000-0000-0000-000000000003",
                ],
                "enable_abstain": false,
                "auto_close": false,
                "create_pdf": false,
            }
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization() {
        let produced: Start = serde_json::from_value(json!({
               "issuer": "00000000-0000-0000-0000-000000000001",
               "parameters": {
                   "initiator_id": "00000000-0000-0000-0000-000000000001",
                   "legal_vote_id": "00000000-0000-0000-0000-000000000002",
                   "start_time":"2025-01-01T00:00:00Z",
                   "max_votes": 5,
                   "kind": "roll_call",
                   "name": "Test Name",
                   "allowed_participants": [
                      "00000000-0000-0000-0000-000000000003",
                   ],
                   "enable_abstain": false,
                   "auto_close": false,
                   "create_pdf": false,
               }
        }))
        .unwrap();

        let expected = Start {
            issuer: UserId::from_u128(1),
            parameters: Parameters {
                initiator_id: ParticipantId::from_u128(1),
                legal_vote_id: LegalVoteId::from_u128(2),
                start_time: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
                max_votes: 5,
                allowed_users: None,
                inner: UserParameters {
                    kind: VoteKind::RollCall,
                    name: Name::try_from("Test Name").unwrap(),
                    subtitle: None,
                    topic: None,
                    allowed_participants: AllowedParticipants::try_from(vec![
                        ParticipantId::from_u128(3),
                    ])
                    .unwrap(),
                    enable_abstain: false,
                    auto_close: false,
                    duration: None,
                    create_pdf: false,
                    timezone: None,
                },
                token: None,
            },
        };

        assert_eq!(produced, expected);
    }
}
