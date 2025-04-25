// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use super::{ResolvedVote, Summary, TimedEvent};

/// The data used to generate a report with typst
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportData {
    pub summary: Summary,
    pub votes: Vec<ResolvedVote>,
    pub events: Vec<TimedEvent>,
}

#[cfg(test)]
pub(crate) mod tests {
    use opentalk_types_common::users::DisplayName;
    use opentalk_types_signaling_legal_vote::{
        cancel::{CancelReason, CustomCancelReason},
        issue::{Issue, OtherIssue, TechnicalIssue, TechnicalIssueKind},
        tally::Tally,
        user_parameters::Duration,
        vote::{LegalVoteId, VoteKind, VoteOption},
    };
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::ReportData;
    use crate::{
        report::data::{
            Event, ResolvedCancel, ResolvedReportedIssue, ResolvedVote, StopReason, Summary,
            TimedEvent,
        },
        storage::v1::FinalResults,
    };

    pub(crate) fn example_live_roll_call() -> ReportData {
        ReportData {
            summary: Summary {
                title: "Weather Vote".into(),
                subtitle: Some("Another one of these weather votes".into()),
                topic: Some("Is the weather good today?".into()),
                kind: VoteKind::LiveRollCall,
                creator: "Alice Adams"
                    .parse()
                    .expect("value must be parsable as DisplayName"),
                id: LegalVoteId::from_u128(0xee621ab4_72f6_4d39_bbc4_dc1b96a606cf),
                start_time: "2025-01-02T03:04:05"
                    .parse()
                    .expect("value must be parsable as ReportDateTime"),
                participant_count: 8,
                duration: Some(Duration::try_from(300).unwrap()),
                enable_abstain: false,
                auto_close: true,
                end_time: Some(
                    "2025-01-02T03:09:05"
                        .parse()
                        .expect("value must be parsable as ReportDateTime"),
                ),
                stop_reason: StopReason::Auto,
                vote_count: 6,
                final_results: Some(FinalResults::Valid(Tally {
                    yes: 4,
                    no: 2,
                    abstain: Some(5),
                })),
                report_timezone: "Europe/Berlin"
                    .parse()
                    .expect("value must be parsable as Timezone"),
            },
            votes: vec![
                ResolvedVote {
                    name: Some(
                        "Alice Adams"
                            .parse()
                            .expect("value must be parsable as DisplayName"),
                    ),
                    token: "aaaaaaaa".into(),
                    option: VoteOption::Yes,
                    time: Some(
                        "2025-01-02T03:04:24"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    ),
                },
                ResolvedVote {
                    name: Some(
                        "Bob Burton"
                            .parse()
                            .expect("value must be parsable as DisplayName"),
                    ),
                    token: "bbbbbbbb".into(),
                    option: VoteOption::No,
                    time: Some(
                        "2025-01-02T03:04:20"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    ),
                },
                ResolvedVote {
                    name: Some(
                        "Charlie Cooper"
                            .parse()
                            .expect("value must be parsable as DisplayName"),
                    ),
                    token: "cccccccc".into(),
                    option: VoteOption::No,
                    time: Some(
                        "2025-01-02T03:04:21"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    ),
                },
                ResolvedVote {
                    name: Some(
                        "Dave Dunn"
                            .parse()
                            .expect("value must be parsable as DisplayName"),
                    ),
                    token: "dddddddd".into(),
                    option: VoteOption::Yes,
                    time: Some(
                        "2025-01-02T03:04:19"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    ),
                },
                ResolvedVote {
                    name: Some(
                        "Erin Eaton"
                            .parse()
                            .expect("value must be parsable as DisplayName"),
                    ),
                    token: "eeeeeeee".into(),
                    option: VoteOption::Yes,
                    time: Some(
                        "2025-01-02T03:06:00"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    ),
                },
                ResolvedVote {
                    name: Some(
                        "George Grump"
                            .parse()
                            .expect("value must be parsable as DisplayName"),
                    ),
                    token: "gggggggg".into(),
                    option: VoteOption::Yes,
                    time: Some(
                        "2025-01-02T03:06:00"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    ),
                },
            ],
            events: vec![TimedEvent {
                time: Some(
                    "2025-01-02T03:04:18"
                        .parse()
                        .expect("value must be parsable as ReportDateTime"),
                ),
                event: Event::Issue(ResolvedReportedIssue {
                    name: Some(
                        "Charlie Cooper"
                            .parse()
                            .expect("value must be parsable as DisplayName"),
                    ),
                    issue: Issue::Technical(TechnicalIssue {
                        kind: TechnicalIssueKind::Screenshare,
                        description: None,
                    }),
                }),
            }],
        }
    }

    fn example_live_roll_call_json() -> serde_json::Value {
        json!({
            "summary": {
                "title": "Weather Vote",
                "subtitle": "Another one of these weather votes",
                "topic": "Is the weather good today?",
                "kind": "live_roll_call",
                "creator": "Alice Adams",
                "id": "ee621ab4-72f6-4d39-bbc4-dc1b96a606cf",
                "start_time": "2025-01-02T03:04:05",
                "participant_count": 8,
                "duration": 300,
                "enable_abstain": false,
                "auto_close": true,
                "end_time": "2025-01-02T03:09:05",
                "stop_reason": {
                    "kind": "auto",
                },
                "vote_count": 6,
                "final_results": {
                    "results": "valid",
                    "yes": 4,
                    "no": 2,
                    "abstain": 5,
                },
                "report_timezone": "Europe/Berlin"
            },
            "votes": [
                {
                    "name": "Alice Adams",
                    "token": "aaaaaaaa",
                    "option": "yes",
                    "time": "2025-01-02T03:04:24"
                },
                {
                    "name": "Bob Burton",
                    "token": "bbbbbbbb",
                    "option": "no",
                    "time": "2025-01-02T03:04:20",
                },
                {
                    "name": "Charlie Cooper",
                    "token": "cccccccc",
                    "option": "no",
                    "time": "2025-01-02T03:04:21",
                },
                {
                    "name": "Dave Dunn",
                    "token": "dddddddd",
                    "option": "yes",
                    "time": "2025-01-02T03:04:19",
                },
                {
                    "name": "Erin Eaton",
                    "token": "eeeeeeee",
                    "option": "yes",
                    "time": "2025-01-02T03:06:00",
                },
                {
                    "name": "George Grump",
                    "token": "gggggggg",
                    "option": "yes",
                    "time": "2025-01-02T03:06:00",
                },

            ],
            "events": [
                {
                    "kind": "issue",
                    "event_details": {
                        "name": "Charlie Cooper",
                        "kind": "screenshare",
                    },
                    "time": "2025-01-02T03:04:18",
                },
            ],
        })
    }

    #[test]
    fn serialize_live_roll_call() {
        assert_eq!(
            json!(example_live_roll_call()),
            example_live_roll_call_json()
        );
    }

    #[test]
    fn deserialize_live_roll_call() {
        assert_eq!(
            serde_json::from_value::<ReportData>(example_live_roll_call_json())
                .expect("value must be deserializable"),
            example_live_roll_call(),
        );
    }

    pub(crate) fn canceled_live_roll_call() -> ReportData {
        let mut report_data = example_live_roll_call();
        report_data.summary.final_results = None;
        report_data.summary.stop_reason = StopReason::Canceled(ResolvedCancel {
            user: DisplayName::from_str_lossy("Bob"),
            reason: CancelReason::Custom(CustomCancelReason::try_from("test").unwrap()),
        });

        report_data
    }

    pub(crate) fn example_roll_call() -> ReportData {
        ReportData {
            summary: Summary {
                title: "End meeting early".into(),
                subtitle: Some("Should we end today's meeting earlier?".into()),
                topic: None,
                kind: VoteKind::RollCall,
                creator: "Alice Adams"
                    .parse()
                    .expect("value must be parsable as DisplayName"),
                id: LegalVoteId::from_u128(0x21ca8797_915a_4255_86d7_23ad0508905f),
                start_time: "2025-02-09T08:41:50"
                    .parse()
                    .expect("value must be parsable as ReportDateTime"),
                participant_count: 4,
                duration: Some(Duration::try_from(60).unwrap()),
                enable_abstain: false,
                auto_close: false,
                end_time: Some(
                    "2025-02-09T08:42:50"
                        .parse()
                        .expect("value must be parsable as ReportDateTime"),
                ),
                stop_reason: StopReason::Expired,
                vote_count: 3,
                final_results: Some(FinalResults::Valid(Tally {
                    yes: 2,
                    no: 1,
                    abstain: None,
                })),
                report_timezone: "Europe/Vienna"
                    .parse()
                    .expect("value must be parsable as Timezone"),
            },
            votes: vec![
                ResolvedVote {
                    name: Some(
                        "Bob Burton"
                            .parse()
                            .expect("value must be parsable as DisplayName"),
                    ),
                    token: "WPFPHL6RH7Y".into(),
                    option: VoteOption::No,
                    time: Some(
                        "2025-02-09T08:41:56"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    ),
                },
                ResolvedVote {
                    name: Some(
                        "Alice Adams"
                            .parse()
                            .expect("value must be parsable as DisplayName"),
                    ),
                    token: "FmrpkqvtHN8".into(),
                    option: VoteOption::Yes,
                    time: Some(
                        "2025-02-09T08:42:00"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    ),
                },
                ResolvedVote {
                    name: Some(
                        "Dave Dunn"
                            .parse()
                            .expect("value must be parsable as DisplayName"),
                    ),
                    token: "538ks7CrBET".into(),
                    option: VoteOption::Yes,
                    time: Some(
                        "2025-02-09T08:42:35"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    ),
                },
            ],
            events: vec![
                TimedEvent {
                    time: Some(
                        "2025-02-09T08:42:16"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    ),
                    event: Event::Issue(ResolvedReportedIssue {
                        name: Some(
                            "Alice Adams"
                                .parse()
                                .expect("value must be parsable as DisplayName"),
                        ),
                        issue: Issue::Other(OtherIssue {
                            description: "Misunderstood the question :-(".to_string(),
                        }),
                    }),
                },
                TimedEvent {
                    time: Some(
                        "2025-02-09T08:42:26"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    ),
                    event: Event::UserLeft(
                        Some(
                            "Charlie Cooper"
                                .parse()
                                .expect("value must be parsable as DisplayName"),
                        )
                        .into(),
                    ),
                },
                TimedEvent {
                    time: Some(
                        "2025-02-09T08:42:28"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    ),
                    event: Event::UserJoined(
                        Some(
                            "Charlie Cooper"
                                .parse()
                                .expect("value must be parsable as DisplayName"),
                        )
                        .into(),
                    ),
                },
            ],
        }
    }

    fn example_roll_call_json() -> serde_json::Value {
        json!({
            "summary": {
                "title": "End meeting early",
                "subtitle": "Should we end today's meeting earlier?",
                "kind": "roll_call",
                "creator": "Alice Adams",
                "id": "21ca8797-915a-4255-86d7-23ad0508905f",
                "start_time": "2025-02-09T08:41:50",
                "participant_count": 4,
                "duration": 60,
                "enable_abstain": false,
                "auto_close": false,
                "end_time": "2025-02-09T08:42:50",
                "stop_reason": {
                    "kind": "expired"
                },
                "vote_count": 3,
                "final_results": {
                    "results": "valid",
                    "yes": 2,
                    "no": 1
                },
                "report_timezone": "Europe/Vienna"
            },
            "votes": [
                {
                    "name": "Bob Burton",
                    "token": "WPFPHL6RH7Y",
                    "option": "no",
                    "time": "2025-02-09T08:41:56"
                },
                {
                    "name": "Alice Adams",
                    "token": "FmrpkqvtHN8",
                    "option": "yes",
                    "time": "2025-02-09T08:42:00"
                },
                {
                    "name": "Dave Dunn",
                    "token": "538ks7CrBET",
                    "option": "yes",
                    "time": "2025-02-09T08:42:35"
                }
            ],
            "events": [
                {
                    "time": "2025-02-09T08:42:16",
                    "kind": "issue",
                    "event_details": {
                      "name": "Alice Adams",
                      "description": "Misunderstood the question :-("
                    }
                },
                {
                    "time": "2025-02-09T08:42:26",
                    "kind": "user_left",
                    "event_details": {
                      "name": "Charlie Cooper"
                    }
                },
                {
                    "time": "2025-02-09T08:42:28",
                    "kind": "user_joined",
                    "event_details": {
                      "name": "Charlie Cooper"
                    }
                }
            ]
        })
    }

    #[test]
    fn serialize_roll_call() {
        assert_eq!(json!(example_roll_call()), example_roll_call_json());
    }

    #[test]
    fn deserialize_roll_call() {
        assert_eq!(
            serde_json::from_value::<ReportData>(example_roll_call_json())
                .expect("value must be deserializable"),
            example_roll_call(),
        );
    }

    pub(crate) fn example_pseudonymous() -> ReportData {
        ReportData {
            summary: Summary {
                title: "Example Pseudonymous Vote".into(),
                subtitle: None,
                topic: None,
                kind: VoteKind::Pseudonymous,
                creator: "Alice Adams"
                    .parse()
                    .expect("value must be parsable as DisplayName"),
                id: LegalVoteId::from_u128(0x6a3525fc_aeef_4d7e_9d76_e41ab2cbe469),
                start_time: "2025-02-08T12:32:09"
                    .parse()
                    .expect("value must be parsable as ReportDateTime"),
                participant_count: 4,
                duration: Some(Duration::try_from(60).unwrap()),
                enable_abstain: true,
                auto_close: true,
                end_time: Some(
                    "2025-02-08T12:32:22"
                        .parse()
                        .expect("value must be parsable as ReportDateTime"),
                ),
                stop_reason: StopReason::Auto,
                vote_count: 4,
                final_results: Some(FinalResults::Valid(Tally {
                    yes: 1,
                    no: 2,
                    abstain: Some(1),
                })),
                report_timezone: "Europe/Vienna"
                    .parse()
                    .expect("value must be parsable as Timezone"),
            },
            votes: vec![
                ResolvedVote {
                    name: None,
                    token: "LPwNXJWs7b1".into(),
                    option: VoteOption::Yes,
                    time: None,
                },
                ResolvedVote {
                    name: None,
                    token: "K5SMSt98f11".into(),
                    option: VoteOption::No,
                    time: None,
                },
                ResolvedVote {
                    name: None,
                    token: "B1yWM5eWQQi".into(),
                    option: VoteOption::Abstain,
                    time: None,
                },
                ResolvedVote {
                    name: None,
                    token: "8PCkuJ9NGoY".into(),
                    option: VoteOption::No,
                    time: None,
                },
            ],
            events: vec![],
        }
    }

    fn example_pseudonymous_json() -> serde_json::Value {
        json!({
            "summary": {
                "title": "Example Pseudonymous Vote",
                "kind": "pseudonymous",
                "creator": "Alice Adams",
                "id": "6a3525fc-aeef-4d7e-9d76-e41ab2cbe469",
                "start_time": "2025-02-08T12:32:09",
                "participant_count": 4,
                "duration": 60,
                "enable_abstain": true,
                "auto_close": true,
                "end_time": "2025-02-08T12:32:22",
                "stop_reason": {
                    "kind": "auto"
                },
                "vote_count": 4,
                "final_results": {
                    "results": "valid",
                    "yes": 1,
                    "no": 2,
                    "abstain": 1
                },
                "report_timezone": "Europe/Vienna"
            },
            "votes": [
                {
                    "token": "LPwNXJWs7b1",
                    "option": "yes"
                },
                {
                    "token": "K5SMSt98f11",
                    "option": "no"
                },
                {
                    "token": "B1yWM5eWQQi",
                    "option": "abstain"
                },
                {
                    "token": "8PCkuJ9NGoY",
                    "option": "no"
                }
            ],
            "events": []
        })
    }

    #[test]
    fn serialize_pseudonymous() {
        assert_eq!(json!(example_pseudonymous()), example_pseudonymous_json());
    }

    #[test]
    fn deserialize_pseudonymous() {
        assert_eq!(
            serde_json::from_value::<ReportData>(example_pseudonymous_json())
                .expect("value must be deserializable"),
            example_pseudonymous(),
        );
    }
}
