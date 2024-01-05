// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use database::DbConnection;
use db_storage::{
    events::{
        shared_folders::{EventSharedFolder, NewEventSharedFolder},
        Event, NewEvent,
    },
    tenants::{get_or_create_tenant_by_oidc_id, OidcTenantId},
};
use pretty_assertions::assert_eq;
use serial_test::serial;
use signaling_core::{
    control::storage::try_init_event,
    module_tester::{ModuleTester, WsMessageOutgoing},
    RedisConnection,
};
use test_util::{TestContext, ROOM_ID, USER_1, USER_2};
use types::{
    common::shared_folder::SharedFolder,
    core::{EventId, RoomId, UserId},
    signaling::{
        control::event::{ControlEvent, JoinSuccess},
        Role,
    },
};

async fn make_event(
    conn: &mut DbConnection,
    redis_conn: &mut RedisConnection,
    user_id: UserId,
    room_id: RoomId,
) -> Event {
    let tenant = get_or_create_tenant_by_oidc_id(conn, &OidcTenantId::from("default".to_string()))
        .await
        .unwrap();
    let event = NewEvent {
        title: "Test Event".into(),
        description: "Test Event".into(),
        room: room_id,
        created_by: user_id,
        updated_by: user_id,
        is_time_independent: false,
        is_all_day: Some(false),
        starts_at: None,
        starts_at_tz: None,
        ends_at: None,
        ends_at_tz: None,
        duration_secs: None,
        is_recurring: Some(false),
        recurrence_pattern: None,
        is_adhoc: false,
        tenant_id: tenant.id,
    }
    .insert(conn)
    .await
    .unwrap();

    try_init_event(redis_conn, room_id, Some(event))
        .await
        .unwrap()
        .unwrap()
}

async fn make_shared_folder(conn: &mut DbConnection, event_id: EventId) -> EventSharedFolder {
    NewEventSharedFolder {
        event_id,
        path: "shared/folder".to_string(),
        write_share_id: "123".to_string(),
        write_url: "https://nextcloud.example.com/s/share123".to_string(),
        write_password: "writepassw0rd".to_string(),
        read_share_id: "456".to_string(),
        read_url: "https://nextcloud.example.com/s/share456".to_string(),
        read_password: "readpassw0rd".to_string(),
    }
    .try_insert(conn)
    .await
    .unwrap()
    .unwrap()
}

#[actix_rt::test]
#[serial]
async fn room_without_event() {
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

    let mut module_tester = ModuleTester::<opentalk_shared_folder::SharedFolder>::new(
        test_ctx.db_ctx.db.clone(),
        test_ctx.authz,
        test_ctx.redis_conn,
        room,
    );

    {
        // join a moderator user
        module_tester
            .join_user(
                USER_1.participant_id,
                user1.clone(),
                Role::Moderator,
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
                // check that no shared folder information is available
                assert!(!module_data.contains_key("shared_folder"));
            }
            _ => panic!(),
        }
    }

    {
        // join a non-moderator user
        module_tester
            .join_user(
                USER_2.participant_id,
                user2.clone(),
                Role::User,
                USER_2.name,
                (),
            )
            .await
            .unwrap();
        let join_success = module_tester
            .receive_ws_message(&USER_2.participant_id)
            .await
            .unwrap();
        match join_success {
            WsMessageOutgoing::Control(ControlEvent::JoinSuccess(JoinSuccess {
                module_data,
                ..
            })) => {
                // check that no shared folder information is available
                assert!(!module_data.contains_key("shared_folder"));
            }
            _ => panic!(),
        }
    }
}

#[actix_rt::test]
#[serial]
async fn room_with_event_but_no_shared_folder() {
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

    let mut conn = test_ctx.db_ctx.db.get_conn().await.unwrap();
    let mut redis_conn = test_ctx.redis_conn.clone();
    make_event(&mut conn, &mut redis_conn, user1.id, room.id).await;

    let mut module_tester = ModuleTester::<opentalk_shared_folder::SharedFolder>::new(
        test_ctx.db_ctx.db.clone(),
        test_ctx.authz,
        test_ctx.redis_conn,
        room,
    );

    {
        // join a moderator user
        module_tester
            .join_user(
                USER_1.participant_id,
                user1.clone(),
                Role::Moderator,
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
                // check that no shared folder information is available
                assert!(!module_data.contains_key("shared_folder"));
            }
            _ => panic!(),
        }
    }

    {
        // join a non-moderator user
        module_tester
            .join_user(
                USER_2.participant_id,
                user2.clone(),
                Role::User,
                USER_2.name,
                (),
            )
            .await
            .unwrap();
        let join_success = module_tester
            .receive_ws_message(&USER_2.participant_id)
            .await
            .unwrap();
        match join_success {
            WsMessageOutgoing::Control(ControlEvent::JoinSuccess(JoinSuccess {
                module_data,
                ..
            })) => {
                // check that no shared folder information is available
                assert!(!module_data.contains_key("shared_folder"));
            }
            _ => panic!(),
        }

        // remove the `joined` message from user 1 ws
        let joined = module_tester
            .receive_ws_message(&USER_1.participant_id)
            .await
            .unwrap();
        match joined {
            WsMessageOutgoing::Control(ControlEvent::Joined(_)) => {}
            _ => panic!(),
        }
    }
}

#[actix_rt::test]
#[serial]
async fn room_with_event_and_shared_folder() {
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

    let mut conn = test_ctx.db_ctx.db.get_conn().await.unwrap();
    let mut redis_conn = test_ctx.redis_conn.clone();
    let event = make_event(&mut conn, &mut redis_conn, user1.id, room.id).await;

    let shared_folder = make_shared_folder(&mut conn, event.id).await;
    let moderator_shared_folder = SharedFolder::from(shared_folder);
    let user_shared_folder = moderator_shared_folder.clone().without_write_access();

    let mut module_tester = ModuleTester::<opentalk_shared_folder::SharedFolder>::new(
        test_ctx.db_ctx.db.clone(),
        test_ctx.authz,
        test_ctx.redis_conn,
        room,
    );

    {
        // join a moderator user
        module_tester
            .join_user(
                USER_1.participant_id,
                user1.clone(),
                Role::Moderator,
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
                // check that no shared folder information is available
                let shared_folder = module_data.get::<SharedFolder>().unwrap();
                assert_eq!(shared_folder, Some(moderator_shared_folder));
            }
            _ => panic!(),
        }
    }

    {
        // join a non-moderator user
        module_tester
            .join_user(
                USER_2.participant_id,
                user2.clone(),
                Role::User,
                USER_2.name,
                (),
            )
            .await
            .unwrap();
        let join_success = module_tester
            .receive_ws_message(&USER_2.participant_id)
            .await
            .unwrap();
        match join_success {
            WsMessageOutgoing::Control(ControlEvent::JoinSuccess(JoinSuccess {
                module_data,
                ..
            })) => {
                // check that no shared folder information is available
                let shared_folder = module_data.get::<SharedFolder>().unwrap();
                assert_eq!(shared_folder, Some(user_shared_folder));
            }
            _ => panic!(),
        }

        // remove the `joined` message from user 1 ws
        let joined = module_tester
            .receive_ws_message(&USER_1.participant_id)
            .await
            .unwrap();
        match joined {
            WsMessageOutgoing::Control(ControlEvent::Joined(_)) => {}
            _ => panic!(),
        }
    }
}
