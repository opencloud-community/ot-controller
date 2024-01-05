// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use opentalk_chat::Chat;
use pretty_assertions::assert_eq;
use serde_json::json;
use serial_test::serial;
use signaling_core::module_tester::{ModuleTester, WsMessageOutgoing};
use test_util::{TestContext, ROOM_ID, USER_1, USER_2};
use types::{
    core::{GroupName, Timestamp},
    signaling::{
        chat::{
            command::{ChatCommand, SendMessage},
            event::{ChatEvent, MessageSent},
            peer_state::ChatPeerState,
            state::ChatState,
            Scope,
        },
        control::{
            event::{ControlEvent, JoinSuccess},
            AssociatedParticipant, Participant,
        },
        Role,
    },
};

#[actix_rt::test]
#[serial]
async fn last_seen_timestamps() {
    let test_ctx = TestContext::new().await;

    let user1 = test_ctx
        .db_ctx
        .create_test_user(
            USER_1.n,
            vec![String::from("group1"), String::from("group2")],
        )
        .await
        .unwrap();
    let user2 = test_ctx
        .db_ctx
        .create_test_user(
            USER_2.n,
            vec![String::from("group1"), String::from("group3")],
        )
        .await
        .unwrap();

    let waiting_room = false;
    let room = test_ctx
        .db_ctx
        .create_test_room(ROOM_ID, user1.id, waiting_room)
        .await
        .unwrap();

    let mut module_tester = ModuleTester::<Chat>::new(
        test_ctx.db_ctx.db.clone(),
        test_ctx.authz,
        test_ctx.redis_conn,
        room,
    );

    {
        // join the first user
        module_tester
            .join_user(
                USER_1.participant_id,
                user1.clone(),
                Role::User,
                USER_1.name,
                (),
            )
            .await
            .unwrap();
        let join_success = module_tester
            .receive_ws_message(&USER_1.participant_id)
            .await
            .unwrap();
        match join_success {
            WsMessageOutgoing::Control(ControlEvent::JoinSuccess(JoinSuccess {
                module_data,
                ..
            })) => {
                // check that last seen timestamps are not set
                let chat_data = module_data.get::<ChatState>().unwrap();
                let json = serde_json::to_value(chat_data).unwrap();
                assert_eq!(
                    json,
                    json!({
                        "groups_history": [
                            {
                                "history": [],
                                "name": "group1",
                            },
                            {
                                "history": [],
                                "name": "group2",
                            },
                        ],
                        "enabled": true,
                        "last_seen_timestamp_global": null,
                        "last_seen_timestamps_private": {},
                        "last_seen_timestamps_group": {},
                        "room_history": [],
                        "private_history": [],
                    })
                );
            }
            _ => panic!(),
        }
    }

    {
        // join another user in order to keep the room alive when the first
        // user leaves and joins the room
        module_tester
            .join_user(USER_2.participant_id, user2, Role::User, USER_2.name, ())
            .await
            .unwrap();
        // discard the received ws join success message, no need to test it here
        module_tester
            .receive_ws_message(&USER_2.participant_id)
            .await
            .unwrap();
    }

    let timestamp_global_raw = "2022-01-01T10:11:12Z";
    let timestamp_group_raw = "2022-01-01T10:11:12Z";
    let timestamp_private_raw = "2023-04-05T06:07:08Z";

    {
        // set global timestamp
        let timestamp: Timestamp =
            DateTime::<Utc>::from(DateTime::parse_from_rfc3339(timestamp_global_raw).unwrap())
                .into();
        let message = ChatCommand::SetLastSeenTimestamp {
            scope: Scope::Global,
            timestamp,
        };
        module_tester
            .send_ws_message(&USER_1.participant_id, message)
            .unwrap();
    }

    {
        // set group timestamp for chat of group1
        let timestamp: Timestamp =
            DateTime::<Utc>::from(DateTime::parse_from_rfc3339(timestamp_group_raw).unwrap())
                .into();
        let message = ChatCommand::SetLastSeenTimestamp {
            scope: Scope::Group(GroupName::from("group1".to_owned())),
            timestamp,
        };
        module_tester
            .send_ws_message(&USER_1.participant_id, message)
            .unwrap();
    }

    {
        // set private timestamp for chat with user2
        let timestamp: Timestamp =
            DateTime::<Utc>::from(DateTime::parse_from_rfc3339(timestamp_private_raw).unwrap())
                .into();
        let message = ChatCommand::SetLastSeenTimestamp {
            scope: Scope::Private(USER_2.participant_id),
            timestamp,
        };
        module_tester
            .send_ws_message(&USER_1.participant_id, message)
            .unwrap();
    }

    // leave and join again with the first user
    module_tester.leave(&USER_1.participant_id).await.unwrap();
    module_tester
        .join_user(USER_1.participant_id, user1, Role::User, USER_1.name, ())
        .await
        .unwrap();

    let rejoin_success = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    // verify that we receive the correct timestamp for group1
    match rejoin_success {
        WsMessageOutgoing::Control(ControlEvent::JoinSuccess(JoinSuccess {
            module_data, ..
        })) => {
            // check own groups
            let chat_data = module_data.get::<ChatState>().unwrap();
            let json = serde_json::to_value(chat_data).unwrap();
            assert_eq!(
                json,
                json!({
                    "enabled": true,
                    "room_history": [],
                    "groups_history": [
                        {
                            "history": [],
                            "name": "group1",
                        },
                        {
                            "history": [],
                            "name": "group2",
                        },
                    ],
                    "private_history": [],
                    "last_seen_timestamp_global": timestamp_global_raw,
                    "last_seen_timestamps_private": {
                        "00000000-0000-0000-0000-000000000002": timestamp_private_raw,
                    },
                    "last_seen_timestamps_group": {
                        "group1": timestamp_group_raw,
                    },
                })
            );
        }
        _ => panic!(),
    }

    module_tester.shutdown().await.unwrap();
}

#[actix_rt::test]
#[serial]
async fn common_groups_on_join() {
    let test_ctx = TestContext::new().await;

    let user1 = test_ctx
        .db_ctx
        .create_test_user(
            USER_1.n,
            vec![String::from("group1"), String::from("group2")],
        )
        .await
        .unwrap();

    let user2 = test_ctx
        .db_ctx
        .create_test_user(
            USER_2.n,
            vec![String::from("group1"), String::from("group3")],
        )
        .await
        .unwrap();

    let waiting_room = false;
    let room = test_ctx
        .db_ctx
        .create_test_room(ROOM_ID, user1.id, waiting_room)
        .await
        .unwrap();

    let mut module_tester = ModuleTester::<Chat>::new(
        test_ctx.db_ctx.db.clone(),
        test_ctx.authz,
        test_ctx.redis_conn,
        room,
    );

    module_tester
        .join_user(USER_1.participant_id, user1, Role::User, USER_1.name, ())
        .await
        .unwrap();

    let join_success1 = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    match join_success1 {
        WsMessageOutgoing::Control(ControlEvent::JoinSuccess(JoinSuccess {
            module_data,
            participants,
            ..
        })) => {
            assert!(participants.is_empty());

            // check own groups
            let chat_data = module_data.get::<ChatState>().unwrap();
            let json = serde_json::to_value(chat_data).unwrap();
            assert_eq!(
                json,
                json!({
                    "enabled": true,
                    "groups_history": [
                        {
                            "history":[],
                            "name":"group1"
                        },
                        {
                            "history":[],
                            "name":"group2"
                        }
                    ],
                    "private_history": [],
                    "room_history": [],
                    "last_seen_timestamp_global": null,
                    "last_seen_timestamps_group": {},
                    "last_seen_timestamps_private": {},
                })
            );
        }
        _ => panic!(),
    }

    module_tester
        .join_user(USER_2.participant_id, user2, Role::User, USER_2.name, ())
        .await
        .unwrap();

    let join_success2 = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    match join_success2 {
        WsMessageOutgoing::Control(ControlEvent::JoinSuccess(JoinSuccess {
            module_data,
            participants,
            ..
        })) => {
            assert_eq!(participants.len(), 1);

            // check common groups here
            let peer_frontend_data = participants[0].module_data.get::<ChatPeerState>().unwrap();
            let json = serde_json::to_value(peer_frontend_data).unwrap();
            assert_eq!(json, json!({"groups":["group1"]}));

            // check own groups
            let chat_data = module_data.get::<ChatState>().unwrap();
            let json = serde_json::to_value(chat_data).unwrap();
            assert_eq!(
                json,
                json!({
                    "enabled": true,
                    "room_history": [],
                    "groups_history": [
                        {
                            "history": [],
                            "name":"group1"
                        },
                        {
                            "history": [],
                            "name": "group3"
                        }
                    ],
                    "private_history": [],
                    "last_seen_timestamp_global": null,
                    "last_seen_timestamps_group": {},
                    "last_seen_timestamps_private": {},
                })
            );
        }
        _ => panic!(),
    }

    module_tester.shutdown().await.unwrap();
}

#[actix_rt::test]
#[serial]
async fn private_chat_history_on_join() {
    let test_ctx = TestContext::new().await;

    let user1 = test_ctx
        .db_ctx
        .create_test_user(USER_1.n, vec![])
        .await
        .unwrap();

    let user2 = test_ctx
        .db_ctx
        .create_test_user(USER_2.n, vec![])
        .await
        .unwrap();

    let waiting_room = false;
    let room = test_ctx
        .db_ctx
        .create_test_room(ROOM_ID, user1.id, waiting_room)
        .await
        .unwrap();

    let mut module_tester = ModuleTester::<Chat>::new(
        test_ctx.db_ctx.db.clone(),
        test_ctx.authz,
        test_ctx.redis_conn,
        room,
    );

    module_tester
        .join_user(
            USER_1.participant_id,
            user1.clone(),
            Role::User,
            USER_1.name,
            (),
        )
        .await
        .unwrap();

    let join_success1 = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    match join_success1 {
        WsMessageOutgoing::Control(ControlEvent::JoinSuccess(JoinSuccess {
            module_data,
            participants,
            ..
        })) => {
            assert!(participants.is_empty());

            // check own groups
            let chat_data = module_data.get::<ChatState>().unwrap();
            let json = serde_json::to_value(chat_data).unwrap();
            assert_eq!(
                json,
                json!({
                    "enabled": true,
                    "groups_history": [],
                    "private_history": [],
                    "room_history": [],
                    "last_seen_timestamp_global": null,
                    "last_seen_timestamps_group": {},
                    "last_seen_timestamps_private": {},
                })
            );
        }
        _ => panic!(),
    }

    module_tester
        .join_user(USER_2.participant_id, user2, Role::User, USER_2.name, ())
        .await
        .unwrap();

    let join_success2 = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    match join_success2 {
        WsMessageOutgoing::Control(ControlEvent::JoinSuccess(JoinSuccess {
            module_data,
            participants,
            ..
        })) => {
            assert_eq!(participants.len(), 1);

            // check common groups here
            let peer_frontend_data = participants[0].module_data.get::<ChatPeerState>().unwrap();
            let json = serde_json::to_value(peer_frontend_data).unwrap();
            assert_eq!(json, json!({"groups":[]}));

            // check own groups
            let chat_data = module_data.get::<ChatState>().unwrap();
            let json = serde_json::to_value(chat_data).unwrap();
            assert_eq!(
                json,
                json!({
                    "enabled": true,
                    "room_history": [],
                    "groups_history": [],
                    "private_history": [],
                    "last_seen_timestamp_global": null,
                    "last_seen_timestamps_group": {},
                    "last_seen_timestamps_private": {},
                })
            );
        }
        _ => panic!(),
    }

    let joined = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    assert!(matches!(
        joined,
        WsMessageOutgoing::Control(ControlEvent::Joined(
            Participant {id, module_data: _}
        )) if id == USER_2.participant_id
    ));

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            ChatCommand::SendMessage(SendMessage {
                content: "Low".into(),
                scope: Scope::Private(USER_2.participant_id),
            }),
        )
        .unwrap();

    for user in [USER_1, USER_2] {
        let private_message = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .unwrap();

        assert!(matches!(
            private_message,
            WsMessageOutgoing::Module(ChatEvent::MessageSent(MessageSent {
                id: _,
                source,
                content,
                scope
            })) if source == USER_1.participant_id
               && scope == Scope::Private(USER_2.participant_id)
               && content == *"Low"
        ));
    }

    module_tester.leave(&USER_1.participant_id).await.unwrap();

    let user1_leave_message = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    assert!(matches!(
        user1_leave_message,
        WsMessageOutgoing::Control(ControlEvent::Left(
            AssociatedParticipant {id}
        )) if id == USER_1.participant_id
    ));

    module_tester
        .join_user(USER_1.participant_id, user1, Role::User, USER_1.name, ())
        .await
        .unwrap();

    let join_again_success = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    match join_again_success {
        WsMessageOutgoing::Control(ControlEvent::JoinSuccess(JoinSuccess {
            module_data, ..
        })) => {
            // check that last seen timestamps are not set
            let chat_state = module_data.get::<ChatState>().unwrap();
            let ChatState {
                enabled: _,
                room_history: _,
                groups_history: _,
                mut private_history,
                last_seen_timestamp_global: _,
                last_seen_timestamps_private: _,
                last_seen_timestamps_group: _,
            } = chat_state.expect("some chat state");
            assert!(private_history.len() == 1);
            let mut correspondence = private_history.pop().unwrap();
            assert_eq!(correspondence.correspondent, USER_2.participant_id);
            assert_eq!(correspondence.history.len(), 1);
            let message = correspondence.history.pop().unwrap();
            assert_eq!(message.content, "Low".to_string());
        }
        _ => panic!(),
    }

    module_tester.shutdown().await.unwrap();
}
