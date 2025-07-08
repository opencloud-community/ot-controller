// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::HashMap, time::Duration};

use chrono::{DateTime, TimeZone, Utc};
use opentalk_db_storage::{
    module_resources::{Filter, ModuleResource},
    users::User,
};
use opentalk_signaling_core::{
    SignalingModule, SignalingModuleError,
    module_tester::{ModuleTester, WsMessageOutgoing},
};
use opentalk_signaling_module_legal_vote::{
    LegalVote,
    storage::{Protocol, v1::ProtocolEntry},
};
use opentalk_test_util::{
    ROOM_ID, TestContext, TestUser, USER_1, USER_2, USERS,
    common::{self, TestContextVolatileStorage},
};
use opentalk_types_common::users::DisplayName;
use opentalk_types_signaling::{ParticipantId, Role};
use opentalk_types_signaling_control::event::ControlEvent;
use opentalk_types_signaling_legal_vote::{
    cancel::{CancelReason, CustomCancelReason},
    command::{Cancel, LegalVoteCommand, Stop, Vote},
    event::{
        Canceled, ErrorKind, FinalResults, GuestParticipants, LegalVoteEvent, Response, Results,
        StopKind, Stopped, VoteFailed, VoteResponse, VoteResults, VoteSuccess, VotingRecord,
    },
    parameters::Parameters,
    state::LegalVoteState,
    tally::Tally,
    token::Token,
    user_parameters::{self, AllowedParticipants, Name, Subtitle, Topic, UserParameters},
    vote::{self, LegalVoteId, VoteKind, VoteOption, VoteState, VoteSummary},
};
use pretty_assertions::assert_eq;
use serde_json::{Value, to_value};
use serial_test::serial;

fn compare_stopped_message_except_for_timestamp(
    actual: WsMessageOutgoing<LegalVote>,
    mut expected: WsMessageOutgoing<LegalVote>,
) -> DateTime<Utc> {
    let timestamp = match actual {
        WsMessageOutgoing::Module(LegalVoteEvent::Stopped(Stopped { end_time, .. })) => end_time,
        _ => panic!("Message type mismatch"),
    };
    match expected {
        WsMessageOutgoing::Module(LegalVoteEvent::Stopped(Stopped {
            ref mut end_time, ..
        })) => *end_time = timestamp,
        _ => panic!("Message type mismatch"),
    };
    assert_eq!(actual, expected);
    timestamp
}

#[actix_rt::test]
#[serial]
async fn basic_vote_roll_call_redis() {
    basic_vote_roll_call(TestContextVolatileStorage::Redis).await
}

#[actix_rt::test]
#[serial]
async fn basic_vote_roll_call_memory() {
    basic_vote_roll_call(TestContextVolatileStorage::Memory).await
}

async fn basic_vote_roll_call(storage: TestContextVolatileStorage) {
    let test_ctx = TestContext::new(storage).await;
    let (mut module_tester, user1, _user2) = common::setup_users::<LegalVote>(&test_ctx, ()).await;
    let mut db_conn = test_ctx.db_ctx.db.get_conn().await.unwrap();

    // Start legal vote as user 1
    let start_parameters = UserParameters {
        kind: VoteKind::RollCall,
        name: Name::try_from("TestVote").unwrap(),
        subtitle: Some(Subtitle::try_from("A subtitle").unwrap()),
        topic: Some(Topic::try_from("Does the test work?").unwrap()),
        allowed_participants: AllowedParticipants::try_from(vec![
            USER_1.participant_id,
            USER_2.participant_id,
        ])
        .unwrap(),
        enable_abstain: false,
        auto_close: false,
        duration: None,
        create_pdf: false,
        timezone: None,
    };

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            LegalVoteCommand::Start(start_parameters.clone()),
        )
        .unwrap();

    // Expect Start response in websocket for user 1
    let (legal_vote_id, user_1_token) =
        if let WsMessageOutgoing::Module(LegalVoteEvent::Started(parameters)) = module_tester
            .receive_ws_message(&USER_1.participant_id)
            .await
            .unwrap()
        {
            assert_eq!(parameters.initiator_id, USER_1.participant_id);
            assert_eq!(parameters.inner, start_parameters);
            assert_eq!(parameters.max_votes, 2);
            assert!(parameters.token.is_some());

            (parameters.legal_vote_id, parameters.token.unwrap())
        } else {
            panic!("Expected Start message")
        };

    // Expect Start response in websocket for user 2
    let user_2_token = if let WsMessageOutgoing::Module(LegalVoteEvent::Started(parameters)) =
        module_tester
            .receive_ws_message(&USER_2.participant_id)
            .await
            .unwrap()
    {
        assert_eq!(parameters.initiator_id, USER_1.participant_id);
        assert_eq!(parameters.inner, start_parameters);
        assert_eq!(parameters.legal_vote_id, legal_vote_id);
        assert_eq!(parameters.max_votes, 2);
        assert!(parameters.token.is_some());

        parameters.token.unwrap()
    } else {
        panic!("Expected Start message")
    };

    // Expect a empty legal_vote with `legal_vote_id` to exist in database
    let module_resource =
        ModuleResource::get(&mut db_conn, Filter::new().with_id(*legal_vote_id.inner()))
            .await
            .unwrap()
            .remove(0);

    assert_eq!(module_resource.id, *legal_vote_id.inner());
    assert_eq!(module_resource.created_by, user1.id);

    let protocol = serde_json::from_value::<Protocol>(module_resource.data).unwrap();

    let protocol_entries =
        serde_json::from_str::<Vec<ProtocolEntry>>(protocol.entries.get()).unwrap();

    assert!(protocol_entries.is_empty());

    // Start casting votes

    // Vote 'Yes' with user 1
    let vote_yes = LegalVoteCommand::Vote(Vote {
        legal_vote_id,
        option: VoteOption::Yes,
        token: user_1_token,
    });

    module_tester
        .send_ws_message(&USER_1.participant_id, vote_yes)
        .unwrap();

    // Expect VoteSuccess
    let vote_response = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    let expected_vote_response = WsMessageOutgoing::Module(LegalVoteEvent::Voted(VoteResponse {
        legal_vote_id,
        response: Response::Success(VoteSuccess {
            vote_option: VoteOption::Yes,
            issuer: USER_1.participant_id,
            consumed_token: user_1_token,
        }),
    }));

    assert_eq!(expected_vote_response, vote_response);

    let mut voters = HashMap::new();
    voters.insert(USER_1.participant_id, VoteOption::Yes);

    // Expect a vote Update message on all participants
    let expected_update = WsMessageOutgoing::Module(LegalVoteEvent::Updated(VoteResults {
        legal_vote_id,
        results: Results {
            tally: Tally {
                yes: 1,
                no: 0,
                abstain: None,
            },
            voting_record: VotingRecord::UserVotes(voters.clone()),
        },
    }));

    for user in USERS {
        let update = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .unwrap();

        assert_eq!(expected_update, update);
    }

    // Vote 'No' with user 2
    let vote_no = LegalVoteCommand::Vote(Vote {
        legal_vote_id,
        option: VoteOption::No,
        token: user_2_token,
    });

    module_tester
        .send_ws_message(&USER_2.participant_id, vote_no)
        .unwrap();

    // Expect VoteSuccess
    let vote_response = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    let expected_vote_response = WsMessageOutgoing::Module(LegalVoteEvent::Voted(VoteResponse {
        legal_vote_id,
        response: Response::Success(VoteSuccess {
            vote_option: VoteOption::No,
            issuer: USER_2.participant_id,
            consumed_token: user_2_token,
        }),
    }));

    assert_eq!(expected_vote_response, vote_response);

    voters.insert(USER_2.participant_id, VoteOption::No);

    // Expect a vote Update message on all participants
    let expected_update = WsMessageOutgoing::Module(LegalVoteEvent::Updated(VoteResults {
        legal_vote_id,
        results: Results {
            tally: Tally {
                yes: 1,
                no: 1,
                abstain: None,
            },
            voting_record: VotingRecord::UserVotes(voters.clone()),
        },
    }));

    for user in USERS {
        let update = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .unwrap();

        assert_eq!(expected_update, update);
    }

    // stop vote
    let stop_vote = LegalVoteCommand::Stop(Stop { legal_vote_id });

    module_tester
        .send_ws_message(&USER_1.participant_id, stop_vote)
        .unwrap();

    let expected_stop_message = WsMessageOutgoing::Module(LegalVoteEvent::Stopped(Stopped {
        legal_vote_id,
        kind: StopKind::ByParticipant(USER_1.participant_id),
        results: FinalResults::Valid(Results {
            tally: Tally {
                yes: 1,
                no: 1,
                abstain: None,
            },
            voting_record: VotingRecord::UserVotes(voters),
        }),
        end_time: Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap(),
    }));

    // expect stop messages for all users
    for user in USERS {
        let stop_message = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .expect("Expected stop message");

        compare_stopped_message_except_for_timestamp(stop_message, expected_stop_message.clone());
    }

    // check the vote protocol
    let module_resource =
        ModuleResource::get(&mut db_conn, Filter::new().with_id(*legal_vote_id.inner()))
            .await
            .unwrap()
            .remove(0);

    assert_eq!(module_resource.id, *legal_vote_id.inner());
    assert_eq!(module_resource.created_by, user1.id);

    let protocol = serde_json::from_value::<Protocol>(module_resource.data).unwrap();

    let protocol_entries =
        serde_json::from_str::<Vec<ProtocolEntry>>(protocol.entries.get()).unwrap();

    assert_eq!(protocol_entries.len(), 5);
}

#[actix_rt::test]
#[serial]
async fn basic_vote_live_roll_call_redis() {
    basic_vote_live_roll_call(TestContextVolatileStorage::Redis).await
}

#[actix_rt::test]
#[serial]
async fn basic_vote_live_roll_call_memory() {
    basic_vote_live_roll_call(TestContextVolatileStorage::Memory).await
}

async fn basic_vote_live_roll_call(storage: TestContextVolatileStorage) {
    let test_ctx = TestContext::new(storage).await;
    let (mut module_tester, user1, _user2) = common::setup_users::<LegalVote>(&test_ctx, ()).await;
    let mut db_conn = test_ctx.db_ctx.db.get_conn().await.unwrap();

    // Start legal vote as user 1
    let start_parameters = UserParameters {
        kind: VoteKind::LiveRollCall,
        name: Name::try_from("TestVote").unwrap(),
        subtitle: Some(Subtitle::try_from("TestVote").unwrap()),
        topic: Some(Topic::try_from("Does the test work?").unwrap()),
        allowed_participants: AllowedParticipants::try_from(vec![
            USER_1.participant_id,
            USER_2.participant_id,
        ])
        .unwrap(),
        enable_abstain: false,
        auto_close: false,
        duration: None,
        create_pdf: false,
        timezone: None,
    };

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            LegalVoteCommand::Start(start_parameters.clone()),
        )
        .unwrap();

    // Expect Start response in websocket for user 1
    let (legal_vote_id, user_1_token) =
        if let WsMessageOutgoing::Module(LegalVoteEvent::Started(parameters)) = module_tester
            .receive_ws_message(&USER_1.participant_id)
            .await
            .unwrap()
        {
            assert_eq!(parameters.initiator_id, USER_1.participant_id);
            assert_eq!(parameters.inner, start_parameters);
            assert_eq!(parameters.max_votes, 2);
            assert!(parameters.token.is_some());

            (parameters.legal_vote_id, parameters.token.unwrap())
        } else {
            panic!("Expected Start message")
        };

    // Expect Start response in websocket for user 2
    let user_2_token = if let WsMessageOutgoing::Module(LegalVoteEvent::Started(parameters)) =
        module_tester
            .receive_ws_message(&USER_2.participant_id)
            .await
            .unwrap()
    {
        assert_eq!(parameters.initiator_id, USER_1.participant_id);
        assert_eq!(parameters.inner, start_parameters);
        assert_eq!(parameters.legal_vote_id, legal_vote_id);
        assert_eq!(parameters.max_votes, 2);
        assert!(parameters.token.is_some());

        parameters.token.unwrap()
    } else {
        panic!("Expected Start message")
    };

    // Expect a empty legal_vote with `legal_vote_id` to exist in database
    let module_resource =
        ModuleResource::get(&mut db_conn, Filter::new().with_id(*legal_vote_id.inner()))
            .await
            .unwrap()
            .remove(0);

    assert_eq!(module_resource.id, *legal_vote_id.inner());
    assert_eq!(module_resource.created_by, user1.id);

    let protocol = serde_json::from_value::<Protocol>(module_resource.data).unwrap();

    let protocol_entries =
        serde_json::from_str::<Vec<ProtocolEntry>>(protocol.entries.get()).unwrap();

    assert!(protocol_entries.is_empty());

    // Start casting votes

    // Vote 'Yes' with user 1
    let vote_yes = LegalVoteCommand::Vote(Vote {
        legal_vote_id,
        option: VoteOption::Yes,
        token: user_1_token,
    });

    module_tester
        .send_ws_message(&USER_1.participant_id, vote_yes)
        .unwrap();

    // Expect VoteSuccess
    let vote_response = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    let expected_vote_response = WsMessageOutgoing::Module(LegalVoteEvent::Voted(VoteResponse {
        legal_vote_id,
        response: Response::Success(VoteSuccess {
            vote_option: VoteOption::Yes,
            issuer: USER_1.participant_id,
            consumed_token: user_1_token,
        }),
    }));

    assert_eq!(expected_vote_response, vote_response);

    let mut voters = HashMap::new();
    voters.insert(USER_1.participant_id, VoteOption::Yes);

    // Expect a vote Update message on all participants
    let expected_update = WsMessageOutgoing::Module(LegalVoteEvent::Updated(VoteResults {
        legal_vote_id,
        results: Results {
            tally: Tally {
                yes: 1,
                no: 0,
                abstain: None,
            },
            voting_record: VotingRecord::UserVotes(voters.clone()),
        },
    }));

    for user in USERS {
        let update = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .unwrap();

        assert_eq!(expected_update, update);
    }

    // Vote 'No' with user 2
    let vote_no = LegalVoteCommand::Vote(Vote {
        legal_vote_id,
        option: VoteOption::No,
        token: user_2_token,
    });

    module_tester
        .send_ws_message(&USER_2.participant_id, vote_no)
        .unwrap();

    // Expect VoteSuccess
    let vote_response = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    let expected_vote_response = WsMessageOutgoing::Module(LegalVoteEvent::Voted(VoteResponse {
        legal_vote_id,
        response: Response::Success(VoteSuccess {
            vote_option: VoteOption::No,
            issuer: USER_2.participant_id,
            consumed_token: user_2_token,
        }),
    }));

    assert_eq!(expected_vote_response, vote_response);

    voters.insert(USER_2.participant_id, VoteOption::No);

    // Expect a vote Update message on all participants
    let expected_update = WsMessageOutgoing::Module(LegalVoteEvent::Updated(VoteResults {
        legal_vote_id,
        results: Results {
            tally: Tally {
                yes: 1,
                no: 1,
                abstain: None,
            },
            voting_record: VotingRecord::UserVotes(voters.clone()),
        },
    }));

    for user in USERS {
        let update = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .unwrap();

        assert_eq!(expected_update, update);
    }

    // stop vote
    let stop_vote = LegalVoteCommand::Stop(Stop { legal_vote_id });

    module_tester
        .send_ws_message(&USER_1.participant_id, stop_vote)
        .unwrap();

    let expected_stop_message = WsMessageOutgoing::Module(LegalVoteEvent::Stopped(Stopped {
        legal_vote_id,
        kind: StopKind::ByParticipant(USER_1.participant_id),
        results: FinalResults::Valid(Results {
            tally: Tally {
                yes: 1,
                no: 1,
                abstain: None,
            },
            voting_record: VotingRecord::UserVotes(voters),
        }),
        end_time: Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap(),
    }));

    // expect stop messages for all users
    for user in USERS {
        let stop_message = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .expect("Expected stop message");

        compare_stopped_message_except_for_timestamp(stop_message, expected_stop_message.clone());
    }

    // check the vote protocol
    let module_resource =
        ModuleResource::get(&mut db_conn, Filter::new().with_id(*legal_vote_id.inner()))
            .await
            .unwrap()
            .remove(0);

    assert_eq!(module_resource.id, *legal_vote_id.inner());
    assert_eq!(module_resource.created_by, user1.id);

    let protocol = serde_json::from_value::<Protocol>(module_resource.data).unwrap();

    let protocol_entries =
        serde_json::from_str::<Vec<ProtocolEntry>>(protocol.entries.get()).unwrap();

    assert_eq!(protocol_entries.len(), 5);
}

#[actix_rt::test]
#[serial]
async fn basic_vote_pseudonymous_redis() {
    basic_vote_pseudonymous(TestContextVolatileStorage::Redis).await
}

#[actix_rt::test]
#[serial]
async fn basic_vote_pseudonymous_memory() {
    basic_vote_pseudonymous(TestContextVolatileStorage::Memory).await
}

async fn basic_vote_pseudonymous(storage: TestContextVolatileStorage) {
    let test_ctx = TestContext::new(storage).await;
    let (mut module_tester, user1, _user2) = common::setup_users::<LegalVote>(&test_ctx, ()).await;
    let mut db_conn = test_ctx.db_ctx.db.get_conn().await.unwrap();

    // Start legal vote as user 1
    let start_parameters = UserParameters {
        kind: VoteKind::Pseudonymous,
        name: Name::try_from("TestVote").unwrap(),
        subtitle: Some(Subtitle::try_from("TestVote").unwrap()),
        topic: Some(Topic::try_from("Does the test work?").unwrap()),
        allowed_participants: AllowedParticipants::try_from(vec![
            USER_1.participant_id,
            USER_2.participant_id,
        ])
        .unwrap(),
        enable_abstain: false,
        auto_close: false,
        duration: None,
        create_pdf: false,
        timezone: None,
    };

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            LegalVoteCommand::Start(start_parameters.clone()),
        )
        .unwrap();

    // Expect Start response in websocket for user 1
    let (legal_vote_id, user_1_token) =
        if let WsMessageOutgoing::Module(LegalVoteEvent::Started(parameters)) = module_tester
            .receive_ws_message(&USER_1.participant_id)
            .await
            .unwrap()
        {
            assert_eq!(parameters.initiator_id, USER_1.participant_id);
            assert_eq!(parameters.inner, start_parameters);
            assert_eq!(parameters.max_votes, 2);
            assert!(parameters.token.is_some());

            (parameters.legal_vote_id, parameters.token.unwrap())
        } else {
            panic!("Expected Start message")
        };

    // Expect Start response in websocket for user 2
    let user_2_token = if let WsMessageOutgoing::Module(LegalVoteEvent::Started(parameters)) =
        module_tester
            .receive_ws_message(&USER_2.participant_id)
            .await
            .unwrap()
    {
        assert_eq!(parameters.initiator_id, USER_1.participant_id);
        assert_eq!(parameters.inner, start_parameters);
        assert_eq!(parameters.legal_vote_id, legal_vote_id);
        assert_eq!(parameters.max_votes, 2);
        assert!(parameters.token.is_some());

        parameters.token.unwrap()
    } else {
        panic!("Expected Start message")
    };

    // Expect a empty legal_vote with `legal_vote_id` to exist in database
    let module_resource =
        ModuleResource::get(&mut db_conn, Filter::new().with_id(*legal_vote_id.inner()))
            .await
            .unwrap()
            .remove(0);

    assert_eq!(module_resource.id, *legal_vote_id.inner());
    assert_eq!(module_resource.created_by, user1.id);

    let protocol = serde_json::from_value::<Protocol>(module_resource.data).unwrap();

    let protocol_entries =
        serde_json::from_str::<Vec<ProtocolEntry>>(protocol.entries.get()).unwrap();

    assert!(protocol_entries.is_empty());

    // Start casting votes

    // Vote 'Yes' with user 1
    let vote_yes = LegalVoteCommand::Vote(Vote {
        legal_vote_id,
        option: VoteOption::Yes,
        token: user_1_token,
    });

    module_tester
        .send_ws_message(&USER_1.participant_id, vote_yes)
        .unwrap();

    // Expect VoteSuccess
    let vote_response = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    let expected_vote_response = WsMessageOutgoing::Module(LegalVoteEvent::Voted(VoteResponse {
        legal_vote_id,
        response: Response::Success(VoteSuccess {
            vote_option: VoteOption::Yes,
            issuer: USER_1.participant_id,
            consumed_token: user_1_token,
        }),
    }));

    assert_eq!(expected_vote_response, vote_response);

    // Vote 'No' with user 2
    let vote_no = LegalVoteCommand::Vote(Vote {
        legal_vote_id,
        option: VoteOption::No,
        token: user_2_token,
    });

    module_tester
        .send_ws_message(&USER_2.participant_id, vote_no)
        .unwrap();

    // Expect VoteSuccess
    let vote_response = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    let expected_vote_response = WsMessageOutgoing::Module(LegalVoteEvent::Voted(VoteResponse {
        legal_vote_id,
        response: Response::Success(VoteSuccess {
            vote_option: VoteOption::No,
            issuer: USER_2.participant_id,
            consumed_token: user_2_token,
        }),
    }));

    assert_eq!(expected_vote_response, vote_response);

    // stop vote
    let stop_vote = LegalVoteCommand::Stop(Stop { legal_vote_id });

    module_tester
        .send_ws_message(&USER_1.participant_id, stop_vote)
        .unwrap();

    let token_votes = HashMap::from_iter(vec![
        (user_1_token, VoteOption::Yes),
        (user_2_token, VoteOption::No),
    ]);

    let expected_stop_message = WsMessageOutgoing::Module(LegalVoteEvent::Stopped(Stopped {
        legal_vote_id,
        kind: StopKind::ByParticipant(USER_1.participant_id),
        results: FinalResults::Valid(Results {
            tally: Tally {
                yes: 1,
                no: 1,
                abstain: None,
            },
            voting_record: VotingRecord::TokenVotes(token_votes),
        }),
        end_time: Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap(),
    }));

    // expect stop messages for all users
    for user in USERS {
        let stop_message = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .expect("Expected stop message");

        compare_stopped_message_except_for_timestamp(stop_message, expected_stop_message.clone());
    }

    // check the vote protocol
    let module_resource =
        ModuleResource::get(&mut db_conn, Filter::new().with_id(*legal_vote_id.inner()))
            .await
            .unwrap()
            .remove(0);

    assert_eq!(module_resource.id, *legal_vote_id.inner());
    assert_eq!(module_resource.created_by, user1.id);

    let protocol = serde_json::from_value::<Protocol>(module_resource.data).unwrap();

    let protocol_entries =
        serde_json::from_str::<Vec<ProtocolEntry>>(protocol.entries.get()).unwrap();

    assert_eq!(protocol_entries.len(), 5);
}

#[actix_rt::test]
#[serial]
async fn hidden_legal_vote_redis() {
    hidden_legal_vote(TestContextVolatileStorage::Redis).await
}

#[actix_rt::test]
#[serial]
async fn hidden_legal_vote_memory() {
    hidden_legal_vote(TestContextVolatileStorage::Memory).await
}

async fn hidden_legal_vote(storage: TestContextVolatileStorage) {
    let test_ctx = TestContext::new(storage).await;
    let (mut module_tester, user1, _user2) = common::setup_users::<LegalVote>(&test_ctx, ()).await;
    let mut db_conn = test_ctx.db_ctx.db.get_conn().await.unwrap();

    // Start legal vote as user 1
    let start_parameters = UserParameters {
        kind: VoteKind::Pseudonymous,
        name: Name::try_from("TestVote").unwrap(),
        subtitle: Some(Subtitle::try_from("TestVote").unwrap()),
        topic: Some(Topic::try_from("Does the test work?").unwrap()),
        allowed_participants: AllowedParticipants::try_from(vec![
            USER_1.participant_id,
            USER_2.participant_id,
        ])
        .unwrap(),
        enable_abstain: false,
        auto_close: false,
        duration: None,
        create_pdf: false,
        timezone: None,
    };

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            LegalVoteCommand::Start(start_parameters.clone()),
        )
        .unwrap();

    // Expect Start response in websocket for user 1
    let (legal_vote_id, user_1_token) =
        if let WsMessageOutgoing::Module(LegalVoteEvent::Started(parameters)) = module_tester
            .receive_ws_message(&USER_1.participant_id)
            .await
            .unwrap()
        {
            assert_eq!(parameters.initiator_id, USER_1.participant_id);
            assert_eq!(parameters.inner, start_parameters);
            assert_eq!(parameters.max_votes, 2);
            assert!(parameters.token.is_some());

            (parameters.legal_vote_id, parameters.token.unwrap())
        } else {
            panic!("Expected Start message")
        };

    // Expect Start response in websocket for user 2
    let user_2_token = if let WsMessageOutgoing::Module(LegalVoteEvent::Started(parameters)) =
        module_tester
            .receive_ws_message(&USER_2.participant_id)
            .await
            .unwrap()
    {
        assert_eq!(parameters.initiator_id, USER_1.participant_id);
        assert_eq!(parameters.inner, start_parameters);
        assert_eq!(parameters.legal_vote_id, legal_vote_id);
        assert_eq!(parameters.max_votes, 2);
        assert!(parameters.token.is_some());

        parameters.token.unwrap()
    } else {
        panic!("Expected Start message")
    };

    // Expect a empty legal_vote with `legal_vote_id` to exist in database
    let module_resource =
        ModuleResource::get(&mut db_conn, Filter::new().with_id(*legal_vote_id.inner()))
            .await
            .unwrap()
            .remove(0);

    assert_eq!(module_resource.id, *legal_vote_id.inner());
    assert_eq!(module_resource.created_by, user1.id);

    let protocol = serde_json::from_value::<Protocol>(module_resource.data).unwrap();

    let protocol_entries =
        serde_json::from_str::<Vec<ProtocolEntry>>(protocol.entries.get()).unwrap();

    assert!(protocol_entries.is_empty());

    // Start casting votes

    // Vote 'Yes' with user 1
    let vote_yes = LegalVoteCommand::Vote(Vote {
        legal_vote_id,
        option: VoteOption::Yes,
        token: user_1_token,
    });

    module_tester
        .send_ws_message(&USER_1.participant_id, vote_yes)
        .unwrap();

    // Expect VoteSuccess
    let vote_response = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    let expected_vote_response = WsMessageOutgoing::Module(LegalVoteEvent::Voted(VoteResponse {
        legal_vote_id,
        response: Response::Success(VoteSuccess {
            vote_option: VoteOption::Yes,
            issuer: USER_1.participant_id,
            consumed_token: user_1_token,
        }),
    }));

    assert_eq!(expected_vote_response, vote_response);

    // Vote 'No' with user 2
    let vote_no = LegalVoteCommand::Vote(Vote {
        legal_vote_id,
        option: VoteOption::No,
        token: user_2_token,
    });

    module_tester
        .send_ws_message(&USER_2.participant_id, vote_no)
        .unwrap();

    // Expect VoteSuccess
    let vote_response = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    let expected_vote_response = WsMessageOutgoing::Module(LegalVoteEvent::Voted(VoteResponse {
        legal_vote_id,
        response: Response::Success(VoteSuccess {
            vote_option: VoteOption::No,
            issuer: USER_2.participant_id,
            consumed_token: user_2_token,
        }),
    }));

    assert_eq!(expected_vote_response, vote_response);

    // stop vote
    let stop_vote = LegalVoteCommand::Stop(Stop { legal_vote_id });

    module_tester
        .send_ws_message(&USER_1.participant_id, stop_vote)
        .unwrap();

    let token_votes = HashMap::from_iter(vec![
        (user_1_token, VoteOption::Yes),
        (user_2_token, VoteOption::No),
    ]);

    let expected_stop_message = WsMessageOutgoing::Module(LegalVoteEvent::Stopped(Stopped {
        legal_vote_id,
        kind: StopKind::ByParticipant(USER_1.participant_id),
        results: FinalResults::Valid(Results {
            tally: Tally {
                yes: 1,
                no: 1,
                abstain: None,
            },
            voting_record: VotingRecord::TokenVotes(token_votes),
        }),
        end_time: Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap(),
    }));

    // expect stop messages for all users
    for user in USERS {
        let stop_message = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .expect("Expected stop message");

        compare_stopped_message_except_for_timestamp(stop_message, expected_stop_message.clone());
    }

    // check the vote protocol
    let module_resource =
        ModuleResource::get(&mut db_conn, Filter::new().with_id(*legal_vote_id.inner()))
            .await
            .unwrap()
            .remove(0);

    assert_eq!(module_resource.id, *legal_vote_id.inner());
    assert_eq!(module_resource.created_by, user1.id);

    // TODO: parse and check the vote entries
    let protocol = serde_json::from_value::<Protocol>(module_resource.data).unwrap();

    let protocol_entries =
        serde_json::from_str::<Vec<ProtocolEntry>>(protocol.entries.get()).unwrap();

    assert_eq!(protocol_entries.len(), 5);
}

#[actix_rt::test]
#[serial]
async fn basic_vote_abstain_redis() {
    basic_vote_abstain(TestContextVolatileStorage::Redis).await
}

#[actix_rt::test]
#[serial]
async fn basic_vote_abstain_memory() {
    basic_vote_abstain(TestContextVolatileStorage::Memory).await
}

async fn basic_vote_abstain(storage: TestContextVolatileStorage) {
    let test_ctx = TestContext::new(storage).await;
    let (mut module_tester, user1, _user2) = common::setup_users::<LegalVote>(&test_ctx, ()).await;
    let mut db_conn = test_ctx.db_ctx.db.get_conn().await.unwrap();

    // Start legal vote as user 1
    let start_parameters = UserParameters {
        kind: VoteKind::RollCall,
        name: Name::try_from("TestVote").unwrap(),
        subtitle: Some(Subtitle::try_from("TestVote").unwrap()),
        topic: Some(Topic::try_from("Does the test work?").unwrap()),
        allowed_participants: AllowedParticipants::try_from(vec![
            USER_1.participant_id,
            USER_2.participant_id,
        ])
        .unwrap(),
        enable_abstain: true,
        auto_close: false,
        duration: None,
        create_pdf: false,
        timezone: None,
    };

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            LegalVoteCommand::Start(start_parameters.clone()),
        )
        .unwrap();

    // Expect Start response in websocket for user 1
    let (legal_vote_id, user_1_token) =
        if let WsMessageOutgoing::Module(LegalVoteEvent::Started(parameters)) = module_tester
            .receive_ws_message(&USER_1.participant_id)
            .await
            .unwrap()
        {
            assert_eq!(parameters.initiator_id, USER_1.participant_id);
            assert_eq!(parameters.inner, start_parameters);
            assert_eq!(parameters.max_votes, 2);
            assert!(parameters.token.is_some());

            (parameters.legal_vote_id, parameters.token.unwrap())
        } else {
            panic!("Expected Start message")
        };

    // Expect Start response in websocket for user 2
    let user_2_token = if let WsMessageOutgoing::Module(LegalVoteEvent::Started(parameters)) =
        module_tester
            .receive_ws_message(&USER_2.participant_id)
            .await
            .unwrap()
    {
        assert_eq!(parameters.initiator_id, USER_1.participant_id);
        assert_eq!(parameters.inner, start_parameters);
        assert_eq!(parameters.legal_vote_id, legal_vote_id);
        assert_eq!(parameters.max_votes, 2);
        assert!(parameters.token.is_some());

        parameters.token.unwrap()
    } else {
        panic!("Expected Start message")
    };

    // Expect a empty legal_vote with `legal_vote_id` to exist in database
    let module_resource =
        ModuleResource::get(&mut db_conn, Filter::new().with_id(*legal_vote_id.inner()))
            .await
            .unwrap()
            .remove(0);

    assert_eq!(module_resource.id, *legal_vote_id.inner());
    assert_eq!(module_resource.created_by, user1.id);

    let protocol = serde_json::from_value::<Protocol>(module_resource.data).unwrap();

    let protocol_entries =
        serde_json::from_str::<Vec<ProtocolEntry>>(protocol.entries.get()).unwrap();

    assert!(protocol_entries.is_empty());

    // Start casting votes

    // Vote 'Abstain' with user 1
    let vote_abstain = LegalVoteCommand::Vote(Vote {
        legal_vote_id,
        option: VoteOption::Abstain,
        token: user_1_token,
    });

    module_tester
        .send_ws_message(&USER_1.participant_id, vote_abstain)
        .unwrap();

    // Expect VoteSuccess
    let vote_response = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    let expected_vote_response = WsMessageOutgoing::Module(LegalVoteEvent::Voted(VoteResponse {
        legal_vote_id,
        response: Response::Success(VoteSuccess {
            vote_option: VoteOption::Abstain,
            issuer: USER_1.participant_id,
            consumed_token: user_1_token,
        }),
    }));

    assert_eq!(expected_vote_response, vote_response);

    let mut voters = HashMap::new();
    voters.insert(USER_1.participant_id, VoteOption::Abstain);

    // Expect a vote Update message on all participants
    let expected_update = WsMessageOutgoing::Module(LegalVoteEvent::Updated(VoteResults {
        legal_vote_id,
        results: Results {
            tally: Tally {
                yes: 0,
                no: 0,
                abstain: Some(1),
            },
            voting_record: VotingRecord::UserVotes(voters.clone()),
        },
    }));

    for user in USERS {
        let update = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .unwrap();

        assert_eq!(expected_update, update);
    }

    // Vote 'No' with user 2
    let vote_no = LegalVoteCommand::Vote(Vote {
        legal_vote_id,
        option: VoteOption::No,
        token: user_2_token,
    });

    module_tester
        .send_ws_message(&USER_2.participant_id, vote_no)
        .unwrap();

    // Expect VoteSuccess
    let vote_response = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    let expected_vote_response = WsMessageOutgoing::Module(LegalVoteEvent::Voted(VoteResponse {
        legal_vote_id,
        response: Response::Success(VoteSuccess {
            vote_option: VoteOption::No,
            issuer: USER_2.participant_id,
            consumed_token: user_2_token,
        }),
    }));

    assert_eq!(expected_vote_response, vote_response);

    voters.insert(USER_2.participant_id, VoteOption::No);

    // Expect a vote Update message on all participants
    let expected_update = WsMessageOutgoing::Module(LegalVoteEvent::Updated(VoteResults {
        legal_vote_id,
        results: Results {
            tally: Tally {
                yes: 0,
                no: 1,
                abstain: Some(1),
            },
            voting_record: VotingRecord::UserVotes(voters.clone()),
        },
    }));

    for user in USERS {
        let update = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .unwrap();

        assert_eq!(expected_update, update);
    }

    // stop vote
    let stop_vote = LegalVoteCommand::Stop(Stop { legal_vote_id });

    module_tester
        .send_ws_message(&USER_1.participant_id, stop_vote)
        .unwrap();

    let expected_stop_message = WsMessageOutgoing::Module(LegalVoteEvent::Stopped(Stopped {
        legal_vote_id,
        kind: StopKind::ByParticipant(USER_1.participant_id),
        results: FinalResults::Valid(Results {
            tally: Tally {
                yes: 0,
                no: 1,
                abstain: Some(1),
            },
            voting_record: VotingRecord::UserVotes(voters),
        }),
        end_time: Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap(),
    }));

    // expect stop messages for all users
    for user in USERS {
        let stop_message = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .expect("Expected stop message");

        compare_stopped_message_except_for_timestamp(stop_message, expected_stop_message.clone());
    }

    // check the vote protocol
    let module_resource =
        ModuleResource::get(&mut db_conn, Filter::new().with_id(*legal_vote_id.inner()))
            .await
            .unwrap()
            .remove(0);

    assert_eq!(module_resource.id, *legal_vote_id.inner());
    assert_eq!(module_resource.created_by, user1.id);

    let protocol = serde_json::from_value::<Protocol>(module_resource.data).unwrap();

    let protocol_entries =
        serde_json::from_str::<Vec<ProtocolEntry>>(protocol.entries.get()).unwrap();

    assert_eq!(protocol_entries.len(), 5);
}

#[actix_rt::test]
#[serial]
async fn expired_vote_redis() {
    expired_vote(TestContextVolatileStorage::Redis).await
}

#[actix_rt::test]
#[serial]
async fn expired_vote_memory() {
    expired_vote(TestContextVolatileStorage::Memory).await
}

async fn expired_vote(storage: TestContextVolatileStorage) {
    let test_ctx = TestContext::new(storage).await;
    let (mut module_tester, user1, _user2) = common::setup_users::<LegalVote>(&test_ctx, ()).await;
    let mut db_conn = test_ctx.db_ctx.db.get_conn().await.unwrap();

    // Start legal vote as user 1
    let start_parameters = UserParameters {
        kind: VoteKind::RollCall,
        name: Name::try_from("TestVote").unwrap(),
        subtitle: Some(Subtitle::try_from("TestVote").unwrap()),
        topic: Some(Topic::try_from("Does the test work?").unwrap()),
        allowed_participants: AllowedParticipants::try_from(vec![
            USER_1.participant_id,
            USER_2.participant_id,
        ])
        .unwrap(),
        enable_abstain: false,
        auto_close: false,
        duration: Some(user_parameters::Duration::try_from(5).unwrap()),
        create_pdf: false,
        timezone: None,
    };

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            LegalVoteCommand::Start(start_parameters.clone()),
        )
        .unwrap();

    // Expect Start response in websocket for user 1
    let legal_vote_id = if let WsMessageOutgoing::Module(LegalVoteEvent::Started(parameters)) =
        module_tester
            .receive_ws_message(&USER_1.participant_id)
            .await
            .unwrap()
    {
        assert_eq!(parameters.initiator_id, USER_1.participant_id);
        assert_eq!(parameters.inner, start_parameters);
        assert_eq!(parameters.max_votes, 2);

        parameters.legal_vote_id
    } else {
        panic!("Expected Start message")
    };

    // Expect Start response in websocket for user 2
    if let WsMessageOutgoing::Module(LegalVoteEvent::Started(parameters)) = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap()
    {
        assert_eq!(parameters.initiator_id, USER_1.participant_id);
        assert_eq!(parameters.inner, start_parameters);
        assert_eq!(parameters.legal_vote_id, legal_vote_id);
        assert_eq!(parameters.max_votes, 2);
    } else {
        panic!("Expected Start message")
    };

    // Expect a empty legal_vote with `legal_vote_id` to exist in database
    let module_resource =
        ModuleResource::get(&mut db_conn, Filter::new().with_id(*legal_vote_id.inner()))
            .await
            .unwrap()
            .remove(0);

    assert_eq!(module_resource.id, *legal_vote_id.inner());
    assert_eq!(module_resource.created_by, user1.id);

    let protocol = serde_json::from_value::<Protocol>(module_resource.data).unwrap();

    let protocol_entries =
        serde_json::from_str::<Vec<ProtocolEntry>>(protocol.entries.get()).unwrap();

    assert!(protocol_entries.is_empty());

    let expected_stop_message = WsMessageOutgoing::Module(LegalVoteEvent::Stopped(Stopped {
        legal_vote_id,
        kind: StopKind::Expired,
        results: FinalResults::Valid(Results {
            tally: Tally {
                yes: 0,
                no: 0,
                abstain: None,
            },
            voting_record: VotingRecord::UserVotes(HashMap::new()),
        }),
        end_time: Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap(),
    }));

    // receive expired stop message on user 1
    let stop_message = module_tester
        .receive_ws_message_override_timeout(&USER_1.participant_id, Duration::from_secs(6))
        .await
        .expect("Didn't receive stop message after 5 seconds, vote should have expired");

    compare_stopped_message_except_for_timestamp(stop_message, expected_stop_message.clone());

    // receive expired stop message on user 2
    let stop_message = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .expect("Didn't receive stop message for user 2");

    compare_stopped_message_except_for_timestamp(stop_message, expected_stop_message.clone());

    // check the vote protocol
    let module_resource =
        ModuleResource::get(&mut db_conn, Filter::new().with_id(*legal_vote_id.inner()))
            .await
            .unwrap()
            .remove(0);

    assert_eq!(module_resource.id, *legal_vote_id.inner());
    assert_eq!(module_resource.created_by, user1.id);

    let protocol = serde_json::from_value::<Protocol>(module_resource.data).unwrap();

    let protocol_entries =
        serde_json::from_str::<Vec<ProtocolEntry>>(protocol.entries.get()).unwrap();

    assert_eq!(protocol_entries.len(), 3);
}

#[actix_rt::test]
#[serial]
async fn auto_stop_vote_redis() {
    auto_stop_vote(TestContextVolatileStorage::Redis).await
}

#[actix_rt::test]
#[serial]
async fn auto_stop_vote_memory() {
    auto_stop_vote(TestContextVolatileStorage::Memory).await
}

async fn auto_stop_vote(storage: TestContextVolatileStorage) {
    let test_ctx = TestContext::new(storage).await;
    let (mut module_tester, user1, _user2) = common::setup_users::<LegalVote>(&test_ctx, ()).await;
    let mut db_conn = test_ctx.db_ctx.db.get_conn().await.unwrap();

    // Start legal vote as user 1
    let start_parameters = UserParameters {
        kind: VoteKind::RollCall,
        name: Name::try_from("TestVote").unwrap(),
        subtitle: Some(Subtitle::try_from("TestVote").unwrap()),
        topic: Some(Topic::try_from("Does the test work?").unwrap()),
        allowed_participants: AllowedParticipants::try_from(vec![
            USER_1.participant_id,
            USER_2.participant_id,
        ])
        .unwrap(),
        enable_abstain: false,
        auto_close: true,
        duration: None,
        create_pdf: false,
        timezone: None,
    };

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            LegalVoteCommand::Start(start_parameters.clone()),
        )
        .unwrap();

    // Expect Start response in websocket for user 1
    let (legal_vote_id, user_1_token) =
        if let WsMessageOutgoing::Module(LegalVoteEvent::Started(parameters)) = module_tester
            .receive_ws_message(&USER_1.participant_id)
            .await
            .unwrap()
        {
            assert_eq!(parameters.initiator_id, USER_1.participant_id);
            assert_eq!(parameters.inner, start_parameters);
            assert_eq!(parameters.max_votes, 2);
            assert!(parameters.token.is_some());

            (parameters.legal_vote_id, parameters.token.unwrap())
        } else {
            panic!("Expected Start message")
        };

    // Expect Start response in websocket for user 2
    let user_2_token = if let WsMessageOutgoing::Module(LegalVoteEvent::Started(parameters)) =
        module_tester
            .receive_ws_message(&USER_2.participant_id)
            .await
            .unwrap()
    {
        assert_eq!(parameters.initiator_id, USER_1.participant_id);
        assert_eq!(parameters.inner, start_parameters);
        assert_eq!(parameters.legal_vote_id, legal_vote_id);
        assert_eq!(parameters.max_votes, 2);
        assert!(parameters.token.is_some());

        parameters.token.unwrap()
    } else {
        panic!("Expected Start message")
    };

    // Expect a empty legal_vote with `legal_vote_id` to exist in database
    let module_resource =
        ModuleResource::get(&mut db_conn, Filter::new().with_id(*legal_vote_id.inner()))
            .await
            .unwrap()
            .remove(0);

    assert_eq!(module_resource.id, *legal_vote_id.inner());
    assert_eq!(module_resource.created_by, user1.id);

    let protocol = serde_json::from_value::<Protocol>(module_resource.data).unwrap();

    let protocol_entries =
        serde_json::from_str::<Vec<ProtocolEntry>>(protocol.entries.get()).unwrap();

    assert!(protocol_entries.is_empty());

    // Start casting votes

    // Vote 'Yes' with user 1
    let vote_yes = LegalVoteCommand::Vote(Vote {
        legal_vote_id,
        option: VoteOption::Yes,
        token: user_1_token,
    });

    module_tester
        .send_ws_message(&USER_1.participant_id, vote_yes)
        .unwrap();

    // Expect VoteSuccess
    let vote_response = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    let expected_vote_response = WsMessageOutgoing::Module(LegalVoteEvent::Voted(VoteResponse {
        legal_vote_id,
        response: Response::Success(VoteSuccess {
            vote_option: VoteOption::Yes,
            issuer: USER_1.participant_id,
            consumed_token: user_1_token,
        }),
    }));

    assert_eq!(expected_vote_response, vote_response);

    let mut voters = HashMap::new();
    voters.insert(USER_1.participant_id, VoteOption::Yes);

    // Expect a vote Update message on all participants
    let expected_update = WsMessageOutgoing::Module(LegalVoteEvent::Updated(VoteResults {
        legal_vote_id,
        results: Results {
            tally: Tally {
                yes: 1,
                no: 0,
                abstain: None,
            },
            voting_record: VotingRecord::UserVotes(voters.clone()),
        },
    }));

    for user in USERS {
        let update = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .unwrap();

        assert_eq!(expected_update, update);
    }

    // Vote 'No' with user 2 (auto stop should happen here)
    let vote_no = LegalVoteCommand::Vote(Vote {
        legal_vote_id,
        option: VoteOption::No,
        token: user_2_token,
    });

    module_tester
        .send_ws_message(&USER_2.participant_id, vote_no)
        .unwrap();

    // Expect VoteSuccess
    let vote_response = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    let expected_vote_response = WsMessageOutgoing::Module(LegalVoteEvent::Voted(VoteResponse {
        legal_vote_id,
        response: Response::Success(VoteSuccess {
            vote_option: VoteOption::No,
            issuer: USER_2.participant_id,
            consumed_token: user_2_token,
        }),
    }));

    assert_eq!(expected_vote_response, vote_response);

    voters.insert(USER_2.participant_id, VoteOption::No);

    // Expect a vote Update message on all participants
    let expected_update = WsMessageOutgoing::Module(LegalVoteEvent::Updated(VoteResults {
        legal_vote_id,
        results: Results {
            tally: Tally {
                yes: 1,
                no: 1,
                abstain: None,
            },
            voting_record: VotingRecord::UserVotes(voters.clone()),
        },
    }));

    for user in USERS {
        let update = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .unwrap();

        assert_eq!(expected_update, update);
    }

    let voting_record = VotingRecord::UserVotes(voters);

    let results = Results {
        tally: Tally {
            yes: 1,
            no: 1,
            abstain: None,
        },
        voting_record: voting_record.clone(),
    };

    let final_results = FinalResults::Valid(results);

    let expected_stop_message = WsMessageOutgoing::Module(LegalVoteEvent::Stopped(Stopped {
        legal_vote_id,
        kind: StopKind::Auto,
        results: final_results,
        end_time: Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap(),
    }));

    // expect stop messages for all users
    for user in USERS {
        let stop_message = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .expect("Expected stop message");

        compare_stopped_message_except_for_timestamp(stop_message, expected_stop_message.clone());
    }

    // check the vote protocol
    let module_resource =
        ModuleResource::get(&mut db_conn, Filter::new().with_id(*legal_vote_id.inner()))
            .await
            .unwrap()
            .remove(0);

    assert_eq!(module_resource.id, *legal_vote_id.inner());
    assert_eq!(module_resource.created_by, user1.id);

    let protocol = serde_json::from_value::<Protocol>(module_resource.data).unwrap();

    let protocol_entries =
        serde_json::from_str::<Vec<ProtocolEntry>>(protocol.entries.get()).unwrap();

    if let Value::Array(entries) = to_value(protocol_entries).unwrap() {
        assert_eq!(entries.len(), 5);
    }

    module_tester.shutdown().await.unwrap();
}

#[actix_rt::test]
#[serial]
async fn start_with_one_participant_redis() {
    start_with_one_participant(TestContextVolatileStorage::Redis).await
}

#[actix_rt::test]
#[serial]
async fn start_with_one_participant_memory() {
    start_with_one_participant(TestContextVolatileStorage::Memory).await
}

async fn start_with_one_participant(storage: TestContextVolatileStorage) {
    let test_ctx = TestContext::new(storage).await;
    let (module_tester, _user1, _user2) = common::setup_users::<LegalVote>(&test_ctx, ()).await;

    // Start legal vote as user 1
    let start_parameters = UserParameters {
        kind: VoteKind::RollCall,
        name: Name::try_from("TestVote").unwrap(),
        subtitle: Some(Subtitle::try_from("A subtitle").unwrap()),
        topic: Some(Topic::try_from("Does the test work?").unwrap()),
        allowed_participants: AllowedParticipants::try_from(vec![USER_1.participant_id]).unwrap(),
        enable_abstain: false,
        auto_close: false,
        duration: None,
        create_pdf: false,
        timezone: None,
    };

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            LegalVoteCommand::Start(start_parameters.clone()),
        )
        .unwrap();

    module_tester.shutdown().await.unwrap()
}

#[actix_rt::test]
#[serial]
async fn initiator_left_redis() {
    initiator_left(TestContextVolatileStorage::Redis).await
}

#[actix_rt::test]
#[serial]
async fn initiator_left_memory() {
    initiator_left(TestContextVolatileStorage::Memory).await
}

async fn initiator_left(storage: TestContextVolatileStorage) {
    let test_ctx = TestContext::new(storage).await;
    let (mut module_tester, _user1, _user2) = common::setup_users::<LegalVote>(&test_ctx, ()).await;

    default_start_setup(&mut module_tester).await;

    // leave with user 1
    module_tester.leave(&USER_1.participant_id).await.unwrap();

    // receive cancel on user 2
    let initiator_left_cancel = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    if let WsMessageOutgoing::Module(LegalVoteEvent::Canceled(Canceled {
        legal_vote_id: _,
        reason,
        end_time: _,
    })) = initiator_left_cancel
    {
        assert_eq!(reason, CancelReason::InitiatorLeft);
    } else {
        panic!("Expected cancel due to initiator leaving")
    }

    module_tester.shutdown().await.unwrap()
}

#[actix_rt::test]
#[serial]
async fn ineligible_voter_redis() {
    ineligible_voter(TestContextVolatileStorage::Redis).await
}

#[actix_rt::test]
#[serial]
async fn ineligible_voter_memory() {
    ineligible_voter(TestContextVolatileStorage::Memory).await
}

async fn ineligible_voter(storage: TestContextVolatileStorage) {
    let test_ctx = TestContext::new(storage).await;
    let (mut module_tester, _user1, _user2) = common::setup_users::<LegalVote>(&test_ctx, ()).await;

    let start_parameters = UserParameters {
        kind: VoteKind::RollCall,
        name: Name::try_from("TestVote").unwrap(),
        subtitle: Some(Subtitle::try_from("TestVote").unwrap()),
        topic: Some(Topic::try_from("Does the test work?").unwrap()),
        allowed_participants: AllowedParticipants::try_from(vec![USER_1.participant_id]).unwrap(),
        enable_abstain: false,
        auto_close: false,
        duration: None,
        create_pdf: false,
        timezone: None,
    };

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            LegalVoteCommand::Start(start_parameters),
        )
        .unwrap();

    let (legal_vote_id, token) = receive_start_on_user2(&mut module_tester).await;
    assert!(token.is_none());

    // try to vote with ineligible user 2
    module_tester
        .send_ws_message(
            &USER_2.participant_id,
            LegalVoteCommand::Vote(Vote {
                legal_vote_id,
                option: VoteOption::Yes,
                token: Token::default(),
            }),
        )
        .unwrap();

    // expect the vote to fail due to the user being ineligible
    let expected_vote_response = WsMessageOutgoing::Module(LegalVoteEvent::Voted(VoteResponse {
        legal_vote_id,
        response: Response::Failed(VoteFailed::Ineligible),
    }));

    let message = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    assert_eq!(expected_vote_response, message);

    module_tester.shutdown().await.unwrap()
}

#[actix_rt::test]
#[serial]
async fn start_with_allowed_guest_redis() {
    start_with_allowed_guest(TestContextVolatileStorage::Redis).await
}

#[actix_rt::test]
#[serial]
async fn start_with_allowed_guest_memory() {
    start_with_allowed_guest(TestContextVolatileStorage::Memory).await
}

async fn start_with_allowed_guest(storage: TestContextVolatileStorage) {
    let test_ctx = TestContext::new(storage).await;
    let (mut module_tester, _user1, _user2) = common::setup_users::<LegalVote>(&test_ctx, ()).await;

    // start the vote with a guest as an allowed participant
    let guest = ParticipantId::from_u128(11311);

    let start_parameters = UserParameters {
        kind: VoteKind::RollCall,
        name: Name::try_from("TestVote").unwrap(),
        subtitle: Some(Subtitle::try_from("TestVote").unwrap()),
        topic: Some(Topic::try_from("Does the test work?").unwrap()),
        allowed_participants: AllowedParticipants::try_from(vec![
            USER_1.participant_id,
            guest,
            USER_2.participant_id,
        ])
        .unwrap(),
        enable_abstain: false,
        auto_close: false,
        duration: None,
        create_pdf: false,
        timezone: None,
    };

    // start vote with user 1
    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            LegalVoteCommand::Start(start_parameters.clone()),
        )
        .unwrap();

    let expected_error = WsMessageOutgoing::Module(LegalVoteEvent::Error(
        ErrorKind::AllowlistContainsGuests(GuestParticipants {
            guests: vec![guest],
        }),
    ));

    let message = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    assert_eq!(expected_error, message);

    module_tester.shutdown().await.unwrap()
}

#[actix_rt::test]
#[serial]
async fn vote_on_nonexistent_vote_redis() {
    vote_on_nonexistent_vote(TestContextVolatileStorage::Redis).await
}

#[actix_rt::test]
#[serial]
async fn vote_on_nonexistent_vote_memory() {
    vote_on_nonexistent_vote(TestContextVolatileStorage::Memory).await
}

async fn vote_on_nonexistent_vote(storage: TestContextVolatileStorage) {
    let test_ctx = TestContext::new(storage).await;
    let (mut module_tester, _user1, _user2) = common::setup_users::<LegalVote>(&test_ctx, ()).await;

    let legal_vote_id = LegalVoteId::from_u128(11311);

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            LegalVoteCommand::Vote(Vote {
                legal_vote_id,
                option: VoteOption::Yes,
                token: Token::new(0),
            }),
        )
        .unwrap();

    let expected_vote_response = WsMessageOutgoing::Module(LegalVoteEvent::Voted(VoteResponse {
        legal_vote_id,
        response: Response::Failed(VoteFailed::InvalidVoteId),
    }));

    let message = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    assert_eq!(expected_vote_response, message);

    module_tester.shutdown().await.unwrap()
}

#[actix_rt::test]
#[serial]
async fn vote_on_completed_vote_redis() {
    vote_on_completed_vote(TestContextVolatileStorage::Redis).await
}

#[actix_rt::test]
#[serial]
async fn vote_on_completed_vote_memory() {
    vote_on_completed_vote(TestContextVolatileStorage::Memory).await
}

async fn vote_on_completed_vote(storage: TestContextVolatileStorage) {
    let test_ctx = TestContext::new(storage).await;
    let (mut module_tester, _user1, _user2) = common::setup_users::<LegalVote>(&test_ctx, ()).await;

    let (legal_vote_id, tokens) = default_start_setup(&mut module_tester).await;

    // stop vote with user 1
    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            LegalVoteCommand::Stop(Stop { legal_vote_id }),
        )
        .unwrap();

    // expect vote stop
    if let WsMessageOutgoing::Module(LegalVoteEvent::Stopped(Stopped { .. })) = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap()
    {
        // seems good
    } else {
        panic!("Expected stop message")
    };

    // try to vote with user 2
    module_tester
        .send_ws_message(
            &USER_2.participant_id,
            LegalVoteCommand::Vote(Vote {
                legal_vote_id,
                option: VoteOption::Yes,
                token: tokens[1].unwrap(),
            }),
        )
        .unwrap();

    let expected_vote_response = WsMessageOutgoing::Module(LegalVoteEvent::Voted(VoteResponse {
        legal_vote_id,
        response: Response::Failed(VoteFailed::InvalidVoteId),
    }));

    let message = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    assert_eq!(expected_vote_response, message);

    module_tester.shutdown().await.unwrap()
}

#[actix_rt::test]
#[serial]
async fn vote_twice_redis() {
    vote_twice(TestContextVolatileStorage::Redis).await
}

#[actix_rt::test]
#[serial]
async fn vote_twice_memory() {
    vote_twice(TestContextVolatileStorage::Memory).await
}

async fn vote_twice(storage: TestContextVolatileStorage) {
    let test_ctx = TestContext::new(storage).await;
    let (mut module_tester, _user1, _user2) = common::setup_users::<LegalVote>(&test_ctx, ()).await;

    let start_parameters = UserParameters {
        kind: VoteKind::RollCall,
        name: Name::try_from("TestVote").unwrap(),
        subtitle: Some(Subtitle::try_from("TestVote").unwrap()),
        topic: Some(Topic::try_from("Does the test work?").unwrap()),
        allowed_participants: AllowedParticipants::try_from(vec![
            USER_1.participant_id,
            USER_2.participant_id,
        ])
        .unwrap(),
        enable_abstain: false,
        auto_close: false,
        duration: None,
        create_pdf: false,
        timezone: None,
    };

    // start vote with user 1
    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            LegalVoteCommand::Start(start_parameters.clone()),
        )
        .unwrap();

    // receive start vote on user 1
    let (legal_vote_id, token) =
        if let WsMessageOutgoing::Module(LegalVoteEvent::Started(Parameters {
            token,
            legal_vote_id,
            ..
        })) = module_tester
            .receive_ws_message(&USER_1.participant_id)
            .await
            .unwrap()
        {
            (legal_vote_id, token.unwrap())
        } else {
            panic!("Expected started message")
        };

    // vote with user 1
    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            LegalVoteCommand::Vote(Vote {
                legal_vote_id,
                option: VoteOption::Yes,
                token,
            }),
        )
        .unwrap();

    let expected_success_vote_response =
        WsMessageOutgoing::Module(LegalVoteEvent::Voted(VoteResponse {
            legal_vote_id,
            response: Response::Success(VoteSuccess {
                vote_option: VoteOption::Yes,
                issuer: USER_1.participant_id,
                consumed_token: token,
            }),
        }));

    let message = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    assert_eq!(expected_success_vote_response, message);

    // vote again with user 1
    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            LegalVoteCommand::Vote(Vote {
                legal_vote_id,
                option: VoteOption::No,
                token,
            }),
        )
        .unwrap();

    let expected_failed_vote_response =
        WsMessageOutgoing::Module(LegalVoteEvent::Voted(VoteResponse {
            legal_vote_id,
            response: Response::Failed(VoteFailed::Ineligible),
        }));

    // Ignore the vote Update message for the first vote above
    module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    let message = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    assert_eq!(expected_failed_vote_response, message);

    module_tester.shutdown().await.unwrap()
}

#[actix_rt::test]
#[serial]
async fn non_moderator_stop_redis() {
    non_moderator_stop(TestContextVolatileStorage::Redis).await
}

#[actix_rt::test]
#[serial]
async fn non_moderator_stop_memory() {
    non_moderator_stop(TestContextVolatileStorage::Memory).await
}

async fn non_moderator_stop(storage: TestContextVolatileStorage) {
    let test_ctx = TestContext::new(storage).await;
    let (mut module_tester, _user1, _user2) = common::setup_users::<LegalVote>(&test_ctx, ()).await;

    let (legal_vote_id, _) = default_start_setup(&mut module_tester).await;

    // stop vote with user 2
    let stop_vote = LegalVoteCommand::Stop(Stop { legal_vote_id });

    module_tester
        .send_ws_message(&USER_2.participant_id, stop_vote)
        .unwrap();

    let expected_error_message =
        WsMessageOutgoing::Module(LegalVoteEvent::Error(ErrorKind::InsufficientPermissions));

    let message = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    assert_eq!(expected_error_message, message);

    module_tester.shutdown().await.unwrap()
}

#[actix_rt::test]
#[serial]
async fn non_moderator_cancel_redis() {
    non_moderator_cancel(TestContextVolatileStorage::Redis).await
}

#[actix_rt::test]
#[serial]
async fn non_moderator_cancel_memory() {
    non_moderator_cancel(TestContextVolatileStorage::Memory).await
}

async fn non_moderator_cancel(storage: TestContextVolatileStorage) {
    let test_ctx = TestContext::new(storage).await;
    let (mut module_tester, _user1, _user2) = common::setup_users::<LegalVote>(&test_ctx, ()).await;

    let (legal_vote_id, _) = default_start_setup(&mut module_tester).await;

    // cancel vote with user 2
    let cancel_vote = LegalVoteCommand::Cancel(Cancel {
        legal_vote_id,
        reason: CustomCancelReason::try_from("Yes").unwrap(),
    });

    module_tester
        .send_ws_message(&USER_2.participant_id, cancel_vote)
        .unwrap();

    let expected_error_message =
        WsMessageOutgoing::Module(LegalVoteEvent::Error(ErrorKind::InsufficientPermissions));

    let message = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    assert_eq!(expected_error_message, message);

    module_tester.shutdown().await.unwrap()
}

#[actix_rt::test]
#[serial]
async fn join_as_guest_redis() {
    join_as_guest(TestContextVolatileStorage::Redis).await
}

#[actix_rt::test]
#[serial]
async fn join_as_guest_memory() {
    join_as_guest(TestContextVolatileStorage::Memory).await
}

async fn join_as_guest(storage: TestContextVolatileStorage) {
    let test_ctx = TestContext::new(storage).await;
    let user1 = test_ctx
        .db_ctx
        .create_test_user(USER_1.n, Vec::new())
        .await
        .unwrap();
    let guest = ParticipantId::from_u128(11311);

    let waiting_room = false;
    let room = test_ctx
        .db_ctx
        .create_test_room(ROOM_ID, user1.id, waiting_room)
        .await
        .unwrap();

    let mut module_tester = ModuleTester::<LegalVote>::new(
        test_ctx.db_ctx.db.clone(),
        test_ctx.authz.clone(),
        test_ctx.volatile.clone(),
        room,
    );

    // Join with guest
    if let Err(error) = module_tester
        .join_guest(guest, &DisplayName::from_str_lossy("Guest"), ())
        .await
    {
        let is_guest_error = matches!(error, SignalingModuleError::NoInitError { .. });
        assert!(is_guest_error, "Module initialized with guest");
    }
    module_tester.shutdown().await.unwrap()
}

#[actix_rt::test]
#[serial]
async fn frontend_data_redis() {
    frontend_data(TestContextVolatileStorage::Redis).await
}

#[actix_rt::test]
#[serial]
async fn frontend_data_memory() {
    frontend_data(TestContextVolatileStorage::Memory).await
}

async fn frontend_data(storage: TestContextVolatileStorage) {
    async fn check_user_join_module_data(
        module_tester: &mut ModuleTester<LegalVote>,
        user3: User,
        frontend_data: LegalVoteState,
    ) {
        // Join with user3
        module_tester
            .join_user(
                USER_3.participant_id,
                user3.clone(),
                Role::User,
                &USER_3.display_name(),
                (),
            )
            .await
            .unwrap();

        // Ignore join message
        module_tester
            .receive_ws_message(&USER_1.participant_id)
            .await
            .unwrap();
        module_tester
            .receive_ws_message(&USER_2.participant_id)
            .await
            .unwrap();

        let WsMessageOutgoing::Control(ControlEvent::JoinSuccess(join_success)) = module_tester
            .receive_ws_message(&USER_3.participant_id)
            .await
            .unwrap()
        else {
            panic!("Expected JoinSuccess Message")
        };
        assert!(join_success.module_data.contains_key(&LegalVote::NAMESPACE));
        assert_eq!(
            join_success.module_data.get::<LegalVoteState>().unwrap(),
            Some(frontend_data)
        );

        module_tester.leave(&USER_3.participant_id).await.unwrap();

        // Ignore leave message
        module_tester
            .receive_ws_message(&USER_1.participant_id)
            .await
            .unwrap();
        module_tester
            .receive_ws_message(&USER_2.participant_id)
            .await
            .unwrap();
    }

    let test_ctx = TestContext::new(storage).await;
    let (mut module_tester, _user1, _user2) = common::setup_users::<LegalVote>(&test_ctx, ()).await;

    const USER_3: TestUser = TestUser {
        n: 3,
        participant_id: ParticipantId::from_u128(3),
        name: "user3",
    };
    let user3 = test_ctx
        .db_ctx
        .create_test_user(USER_3.n, vec![])
        .await
        .unwrap();

    // Expect empty votes on join
    check_user_join_module_data(
        &mut module_tester,
        user3.clone(),
        LegalVoteState { votes: vec![] },
    )
    .await;

    // Start legal vote as user 1
    let start_parameters = UserParameters {
        kind: VoteKind::RollCall,
        name: Name::try_from("TestVote").unwrap(),
        subtitle: Some(Subtitle::try_from("TestVote").unwrap()),
        topic: Some(Topic::try_from("Does the test work?").unwrap()),
        allowed_participants: AllowedParticipants::try_from(vec![
            USER_1.participant_id,
            USER_2.participant_id,
        ])
        .unwrap(),
        enable_abstain: false,
        auto_close: true,
        duration: None,
        create_pdf: false,
        timezone: None,
    };

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            LegalVoteCommand::Start(start_parameters.clone()),
        )
        .unwrap();

    // Expect Start response in websocket for user 1
    let (parameters, user_1_token) =
        if let WsMessageOutgoing::Module(LegalVoteEvent::Started(parameters)) = module_tester
            .receive_ws_message(&USER_1.participant_id)
            .await
            .unwrap()
        {
            let token = parameters.token.unwrap();
            (parameters, token)
        } else {
            panic!("Expected Start message")
        };

    // Expect Start response in websocket for user 2
    let user_2_token = if let WsMessageOutgoing::Module(LegalVoteEvent::Started(parameters)) =
        module_tester
            .receive_ws_message(&USER_2.participant_id)
            .await
            .unwrap()
    {
        parameters.token.unwrap()
    } else {
        panic!("Expected Start message")
    };

    // Check frontend_data
    {
        let mut parameters = parameters.clone();
        parameters.token = None;
        check_user_join_module_data(
            &mut module_tester,
            user3.clone(),
            LegalVoteState {
                votes: vec![VoteSummary {
                    parameters,
                    state: VoteState::Started,
                    end_time: None,
                }],
            },
        )
        .await;
    }

    let legal_vote_id = parameters.legal_vote_id;

    // Start casting votes

    // Vote 'Yes' with user 1
    let vote_yes = LegalVoteCommand::Vote(Vote {
        legal_vote_id,
        option: VoteOption::Yes,
        token: user_1_token,
    });

    module_tester
        .send_ws_message(&USER_1.participant_id, vote_yes)
        .unwrap();

    // Ignore VoteSuccess
    module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    // Vote 'No' with user 2 (auto stop should happen here)
    let vote_no = LegalVoteCommand::Vote(Vote {
        legal_vote_id,
        option: VoteOption::No,
        token: user_2_token,
    });

    module_tester
        .send_ws_message(&USER_2.participant_id, vote_no)
        .unwrap();

    // Ignore VoteSuccess
    module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    // Ignore the vote Update messages on all participants for the two votes above
    for user in USERS {
        module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .unwrap();
        module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .unwrap();
    }

    // Check stop messages for all users and extract timestamp
    let mut timestamp = None;
    for user in USERS {
        let stop_message = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .expect("Expected stop message");
        if let WsMessageOutgoing::Module(LegalVoteEvent::Stopped(Stopped { end_time, .. })) =
            stop_message
        {
            timestamp = Some(end_time);
        } else {
            panic!("Stop message expected");
        }
    }
    assert!(timestamp.is_some());

    let vote_1_summary = {
        let mut parameters = parameters.clone();
        parameters.token = None;
        VoteSummary {
            parameters,
            state: VoteState::Finished {
                stop_kind: vote::StopKind::Auto,
                results: Results {
                    tally: Tally {
                        yes: 1,
                        no: 1,
                        abstain: None,
                    },
                    voting_record: VotingRecord::UserVotes(HashMap::from_iter([
                        (USER_1.participant_id, VoteOption::Yes),
                        (USER_2.participant_id, VoteOption::No),
                    ])),
                },
            },
            end_time: timestamp,
        }
    };

    // Check frontend_data upon user 3 rejoin
    check_user_join_module_data(
        &mut module_tester,
        user3.clone(),
        LegalVoteState {
            votes: vec![vote_1_summary.clone()],
        },
    )
    .await;

    // Start legal vote as user 1
    let start_parameters = UserParameters {
        kind: VoteKind::Pseudonymous,
        name: Name::try_from("TestVote pseudonymous").unwrap(),
        subtitle: Some(Subtitle::try_from("TestVote").unwrap()),
        topic: Some(Topic::try_from("Does the test work?").unwrap()),
        allowed_participants: AllowedParticipants::try_from(vec![
            USER_1.participant_id,
            USER_2.participant_id,
        ])
        .unwrap(),
        enable_abstain: true,
        auto_close: true,
        duration: None,
        create_pdf: false,
        timezone: None,
    };

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            LegalVoteCommand::Start(start_parameters.clone()),
        )
        .unwrap();

    // Expect Started event in websocket for user 1
    let parameters = if let WsMessageOutgoing::Module(LegalVoteEvent::Started(parameters)) =
        module_tester
            .receive_ws_message(&USER_1.participant_id)
            .await
            .unwrap()
    {
        parameters
    } else {
        panic!("Expected Start message")
    };

    // Expect Start event in websocket for user 2
    if let WsMessageOutgoing::Module(LegalVoteEvent::Started(_)) = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap()
    {
    } else {
        panic!("Expected Start message")
    };

    // Check frontend_data
    {
        let mut parameters = parameters.clone();
        parameters.token = None;
        check_user_join_module_data(
            &mut module_tester,
            user3.clone(),
            LegalVoteState {
                votes: vec![
                    vote_1_summary,
                    VoteSummary {
                        parameters,
                        state: VoteState::Started,
                        end_time: None,
                    },
                ],
            },
        )
        .await;
    }

    module_tester.shutdown().await.unwrap();
}

/// Start a vote with user1 with default UserParameters
async fn default_vote_start_by_user1(
    module_tester: &mut ModuleTester<LegalVote>,
) -> (LegalVoteId, Option<Token>) {
    let start_parameters = UserParameters {
        kind: VoteKind::RollCall,
        name: Name::try_from("TestVote").unwrap(),
        subtitle: Some(Subtitle::try_from("TestVote").unwrap()),
        topic: Some(Topic::try_from("Does the test work?").unwrap()),
        allowed_participants: AllowedParticipants::try_from(vec![
            USER_1.participant_id,
            USER_2.participant_id,
        ])
        .unwrap(),
        enable_abstain: false,
        auto_close: false,
        duration: None,
        create_pdf: false,
        timezone: None,
    };

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            LegalVoteCommand::Start(start_parameters),
        )
        .unwrap();

    if let WsMessageOutgoing::Module(LegalVoteEvent::Started(Parameters {
        token,
        legal_vote_id,
        ..
    })) = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap()
    {
        (legal_vote_id, token)
    } else {
        panic!("Expected started message")
    }
}

/// Receive the vote start on user2 and return the corresponding vote id
async fn receive_start_on_user2(
    module_tester: &mut ModuleTester<LegalVote>,
) -> (LegalVoteId, Option<Token>) {
    if let WsMessageOutgoing::Module(LegalVoteEvent::Started(Parameters {
        token,
        legal_vote_id,
        ..
    })) = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap()
    {
        (legal_vote_id, token)
    } else {
        panic!("Expected started message")
    }
}

/// A default setup where user1 starts the vote and user2 receives the started response.
///
/// Returns a tuple with the first element being the legal vote id, and the secend element
/// being a vector containing the tokens for user 1 and user 2.
async fn default_start_setup(
    module_tester: &mut ModuleTester<LegalVote>,
) -> (LegalVoteId, Vec<Option<Token>>) {
    let (legal_vote_id, user_1_token) = default_vote_start_by_user1(module_tester).await;
    let (_, user_2_token) = receive_start_on_user2(module_tester).await;
    (legal_vote_id, vec![user_1_token, user_2_token])
}
