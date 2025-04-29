// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_types_common::users::UserId;

use crate::storage::v1::{
    Cancel, FinalResults, MaybeUserInfo, ReportedIssue, Start, StopKind, Vote,
};

/// An event related to an active vote.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    redis_args::ToRedisArgs,
    redis_args::FromRedisValue,
)]
#[serde(rename_all = "snake_case", tag = "event")]
#[from_redis_value(serde)]
#[to_redis_args(serde)]
pub enum VoteEvent {
    /// The vote has started.
    Start(Start),

    /// A vote has been cast.
    Vote(Vote),

    /// The vote has been stopped.
    Stop(StopKind),

    /// The final results of the vote.
    FinalResults(FinalResults),

    /// An issue has been reported.
    Issue(ReportedIssue),

    /// A user has left the room.
    UserLeft(MaybeUserInfo),

    /// A user has joined the room.
    UserJoined(MaybeUserInfo),

    /// The vote has been canceled.
    Cancel(Cancel),
}

impl VoteEvent {
    /// Retrieves the user IDs referenced in the event.
    ///
    /// This method returns a set of user IDs involved in the event (e.g., who voted, reported issues, etc.).
    pub fn get_referenced_user_ids(&self) -> BTreeSet<UserId> {
        match self {
            VoteEvent::Start(start) => start.get_referenced_user_ids(),
            VoteEvent::Vote(vote) => vote.get_referenced_user_ids(),
            VoteEvent::Stop(stop_kind) => stop_kind.get_referenced_user_ids(),
            VoteEvent::FinalResults(final_results) => final_results.get_referenced_user_ids(),
            VoteEvent::Issue(reported_issue) => reported_issue.get_referenced_user_ids(),
            VoteEvent::UserLeft(maybe_user_info) => maybe_user_info.get_referenced_user_ids(),
            VoteEvent::UserJoined(maybe_user_info) => maybe_user_info.get_referenced_user_ids(),
            VoteEvent::Cancel(cancel) => cancel.get_referenced_user_ids(),
        }
    }
}

#[cfg(test)]
mod serde_tests {

    use std::str::FromStr;

    use chrono::{TimeZone, Utc};
    use opentalk_types_signaling::ParticipantId;
    use opentalk_types_signaling_legal_vote::{
        cancel::CancelReason,
        issue::{Issue, TechnicalIssue, TechnicalIssueKind},
        parameters::Parameters,
        tally::Tally,
        token::Token,
        user_parameters::{AllowedParticipants, Name, UserParameters},
        vote::{LegalVoteId, VoteKind, VoteOption},
    };
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::storage::v1::UserInfo;

    #[test]
    fn serialization_start_vote_event() {
        let produced = serde_json::to_value(VoteEvent::Start(Start {
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
        }))
        .unwrap();

        let expected = json!({
            "event": "start",
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
    fn deserialization_start_vote_event() {
        let produced: VoteEvent = serde_json::from_value(json!({
            "event": "start",
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

        let expected = VoteEvent::Start(Start {
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
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialization_vote_vote_event() {
        let produced = serde_json::to_value(VoteEvent::Vote(Vote {
            user_info: Some(UserInfo {
                issuer: UserId::from_u128(1),
                participant_id: ParticipantId::from_u128(2),
            }),
            token: Token::from_str("1111Cn8eVZg").unwrap(),
            option: VoteOption::Yes,
        }))
        .unwrap();

        let expected = json!({
            "event": "vote",
            "issuer": "00000000-0000-0000-0000-000000000001",
            "participant_id": "00000000-0000-0000-0000-000000000002",
            "token": "1111Cn8eVZg",
            "option": "yes",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization_vote_vote_event() {
        let produced: VoteEvent = serde_json::from_value(json!({
            "event": "vote",
            "issuer": "00000000-0000-0000-0000-000000000001",
            "participant_id": "00000000-0000-0000-0000-000000000002",
            "token": "1111Cn8eVZg",
            "option": "yes",
        }))
        .unwrap();

        let expected = VoteEvent::Vote(Vote {
            user_info: Some(UserInfo {
                issuer: UserId::from_u128(1),
                participant_id: ParticipantId::from_u128(2),
            }),
            token: Token::from_str("1111Cn8eVZg").unwrap(),
            option: VoteOption::Yes,
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialization_stop_vote_event() {
        let produced =
            serde_json::to_value(VoteEvent::Stop(StopKind::ByUser(UserId::from_u128(1)))).unwrap();

        let expected = json!({
            "event": "stop",
            "by_user": "00000000-0000-0000-0000-000000000001",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization_stop_vote_event() {
        let produced: VoteEvent = serde_json::from_value(json!({
            "event": "stop",
            "by_user": "00000000-0000-0000-0000-000000000001",
        }))
        .unwrap();

        let expected = VoteEvent::Stop(StopKind::ByUser(UserId::from_u128(1)));

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialization_final_results_vote_event() {
        let produced = serde_json::to_value(VoteEvent::FinalResults(FinalResults::Valid(Tally {
            yes: 1,
            no: 0,
            abstain: None,
        })))
        .unwrap();

        let expected = json!({
            "event": "final_results",
            "results": "valid",
            "yes": 1,
            "no": 0,
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization_final_results_vote_event() {
        let produced: VoteEvent = serde_json::from_value(json!({
            "event": "final_results",
            "results": "valid",
            "yes": 1,
            "no": 0,
        }))
        .unwrap();

        let expected = VoteEvent::FinalResults(FinalResults::Valid(Tally {
            yes: 1,
            no: 0,
            abstain: None,
        }));

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialization_issue_vote_event() {
        let produced = serde_json::to_value(VoteEvent::Issue(ReportedIssue {
            user_info: None,
            issue: Issue::Technical(TechnicalIssue {
                kind: TechnicalIssueKind::Audio,
                description: None,
            }),
        }))
        .unwrap();

        let expected = json!({
            "event": "issue",
            "kind": "audio",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization_issue_vote_event() {
        let produced: VoteEvent = serde_json::from_value(json!({
            "event": "issue",
            "kind": "audio",
        }))
        .unwrap();

        let expected = VoteEvent::Issue(ReportedIssue {
            user_info: None,
            issue: Issue::Technical(TechnicalIssue {
                kind: TechnicalIssueKind::Audio,
                description: None,
            }),
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialization_user_left_vote_event() {
        let produced =
            serde_json::to_value(VoteEvent::UserLeft(MaybeUserInfo { inner: None })).unwrap();

        let expected = json!({
            "event": "user_left",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization_user_left_vote_event() {
        let produced: VoteEvent = serde_json::from_value(json!({
            "event": "user_left",
        }))
        .unwrap();

        let expected = VoteEvent::UserLeft(MaybeUserInfo { inner: None });

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialization_user_joined_vote_event() {
        let produced =
            serde_json::to_value(VoteEvent::UserJoined(MaybeUserInfo { inner: None })).unwrap();

        let expected = json!({
            "event": "user_joined",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization_user_joined_vote_event() {
        let produced: VoteEvent = serde_json::from_value(json!({
            "event": "user_joined",
        }))
        .unwrap();

        let expected = VoteEvent::UserJoined(MaybeUserInfo { inner: None });

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialization_cancel_vote_event() {
        let produced = serde_json::to_value(VoteEvent::Cancel(Cancel {
            issuer: UserId::from_u128(1),
            reason: CancelReason::InitiatorLeft,
        }))
        .unwrap();

        let expected = json!({
            "event": "cancel",
            "issuer": "00000000-0000-0000-0000-000000000001",
            "reason": "initiator_left",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization_cancel_vote_event() {
        let produced: VoteEvent = serde_json::from_value(json!({
            "event": "cancel",
            "issuer": "00000000-0000-0000-0000-000000000001",
            "reason": "initiator_left",
        }))
        .unwrap();

        let expected = VoteEvent::Cancel(Cancel {
            issuer: UserId::from_u128(1),
            reason: CancelReason::InitiatorLeft,
        });

        assert_eq!(produced, expected);
    }
}
