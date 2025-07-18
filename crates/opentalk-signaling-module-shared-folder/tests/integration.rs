// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_database::DbConnection;
use opentalk_db_storage::{
    events::{
        Event, NewEvent,
        shared_folders::{EventSharedFolder, NewEventSharedFolder},
    },
    tenants::{OidcTenantId, get_or_create_tenant_by_oidc_id},
};
use opentalk_signaling_core::{
    VolatileStorage,
    control::ControlStorageProvider as _,
    module_tester::{ModuleTester, WsMessageOutgoing},
};
use opentalk_test_util::{ROOM_ID, TestContext, USER_1, USER_2};
use opentalk_types_common::{
    events::EventId,
    rooms::RoomId,
    shared_folders::{MODULE_ID, SharedFolder},
    users::UserId,
};
use opentalk_types_signaling::Role;
use opentalk_types_signaling_control::event::ControlEvent;
use pretty_assertions::assert_eq;
use serial_test::serial;

async fn make_event(
    conn: &mut DbConnection,
    volatile: &mut VolatileStorage,
    user_id: UserId,
    room_id: RoomId,
) -> Event {
    let tenant = get_or_create_tenant_by_oidc_id(conn, &OidcTenantId::from("default".to_string()))
        .await
        .unwrap();
    let event = NewEvent {
        title: "Test Event".parse().expect("valid event title"),
        description: "Test Event".parse().expect("valid event description"),
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
        show_meeting_details: false,
    }
    .insert(conn)
    .await
    .unwrap();

    volatile
        .control_storage()
        .try_init_event(room_id, Some(event))
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
    let test_ctx = TestContext::default().await;

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

    let mut module_tester =
        ModuleTester::<opentalk_signaling_module_shared_folder::SharedFolder>::new(
            test_ctx.db_ctx.db.clone(),
            test_ctx.authz,
            test_ctx.volatile,
            room,
        );

    {
        // join a moderator user
        module_tester
            .join_user(
                USER_1.participant_id,
                user1.clone(),
                Role::Moderator,
                &USER_1.display_name(),
                (),
            )
            .await
            .unwrap();
        let join_success = module_tester
            .receive_ws_message(&USER_1.participant_id)
            .await
            .unwrap();
        match join_success {
            WsMessageOutgoing::Control(ControlEvent::JoinSuccess(join_success)) => {
                // check that no shared folder information is available
                assert!(!join_success.module_data.contains_key(&MODULE_ID));
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
                &USER_2.display_name(),
                (),
            )
            .await
            .unwrap();
        let join_success = module_tester
            .receive_ws_message(&USER_2.participant_id)
            .await
            .unwrap();
        match join_success {
            WsMessageOutgoing::Control(ControlEvent::JoinSuccess(join_success)) => {
                // check that no shared folder information is available
                assert!(!join_success.module_data.contains_key(&MODULE_ID));
            }
            _ => panic!(),
        }
    }
}

#[actix_rt::test]
#[serial]
async fn room_with_event_but_no_shared_folder() {
    let test_ctx = TestContext::default().await;

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
    let mut volatile = test_ctx.volatile.clone();
    make_event(&mut conn, &mut volatile, user1.id, room.id).await;

    let mut module_tester =
        ModuleTester::<opentalk_signaling_module_shared_folder::SharedFolder>::new(
            test_ctx.db_ctx.db.clone(),
            test_ctx.authz,
            test_ctx.volatile,
            room,
        );

    {
        // join a moderator user
        module_tester
            .join_user(
                USER_1.participant_id,
                user1.clone(),
                Role::Moderator,
                &USER_1.display_name(),
                (),
            )
            .await
            .unwrap();
        let join_success = module_tester
            .receive_ws_message(&USER_1.participant_id)
            .await
            .unwrap();
        match join_success {
            WsMessageOutgoing::Control(ControlEvent::JoinSuccess(join_success)) => {
                // check that no shared folder information is available
                assert!(!join_success.module_data.contains_key(&MODULE_ID));
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
                &USER_2.display_name(),
                (),
            )
            .await
            .unwrap();
        let join_success = module_tester
            .receive_ws_message(&USER_2.participant_id)
            .await
            .unwrap();
        match join_success {
            WsMessageOutgoing::Control(ControlEvent::JoinSuccess(join_success)) => {
                // check that no shared folder information is available
                assert!(!join_success.module_data.contains_key(&MODULE_ID));
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
    let test_ctx = TestContext::default().await;

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
    let mut volatile = test_ctx.volatile.clone();
    let event = make_event(&mut conn, &mut volatile, user1.id, room.id).await;

    let shared_folder = make_shared_folder(&mut conn, event.id).await;
    let moderator_shared_folder = SharedFolder::from(shared_folder);
    let user_shared_folder = moderator_shared_folder.clone().without_write_access();

    let mut module_tester =
        ModuleTester::<opentalk_signaling_module_shared_folder::SharedFolder>::new(
            test_ctx.db_ctx.db.clone(),
            test_ctx.authz,
            test_ctx.volatile,
            room,
        );

    {
        // join a moderator user
        module_tester
            .join_user(
                USER_1.participant_id,
                user1.clone(),
                Role::Moderator,
                &USER_1.display_name(),
                (),
            )
            .await
            .unwrap();
        let join_success = module_tester
            .receive_ws_message(&USER_1.participant_id)
            .await
            .unwrap();
        match join_success {
            WsMessageOutgoing::Control(ControlEvent::JoinSuccess(join_success)) => {
                // check that no shared folder information is available
                let shared_folder = join_success.module_data.get::<SharedFolder>().unwrap();
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
                &USER_2.display_name(),
                (),
            )
            .await
            .unwrap();
        let join_success = module_tester
            .receive_ws_message(&USER_2.participant_id)
            .await
            .unwrap();
        match join_success {
            WsMessageOutgoing::Control(ControlEvent::JoinSuccess(join_success)) => {
                // check that no shared folder information is available
                let shared_folder = join_success.module_data.get::<SharedFolder>().unwrap();
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
