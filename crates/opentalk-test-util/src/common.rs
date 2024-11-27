// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use kustos::Authz;
use opentalk_db_storage::users::User;
use opentalk_signaling_core::{
    module_tester::{ModuleTester, WsMessageOutgoing},
    SignalingModule, VolatileStaticMemoryStorage, VolatileStorage,
};
use opentalk_types_common::{rooms::RoomId, users::DisplayName};
use opentalk_types_signaling::{ParticipantId, Role};
use opentalk_types_signaling_control::event::ControlEvent;
use pretty_assertions::assert_eq;
use snafu::{ResultExt, Whatever};
use tokio::sync::broadcast::Sender;

use crate::{database::DatabaseContext, redis};

#[derive(Debug)]
pub struct TestUser {
    pub n: u32,
    pub participant_id: ParticipantId,
    pub name: &'static str,
}

impl TestUser {
    pub fn display_name(&self) -> DisplayName {
        DisplayName::from_str_lossy(self.name)
    }
}

pub const ROOM_ID: RoomId = RoomId::from_u128(2000);

pub const USER_1: TestUser = TestUser {
    n: 1,
    participant_id: ParticipantId::from_u128(1),
    name: "user1",
};

pub const USER_2: TestUser = TestUser {
    n: 2,
    participant_id: ParticipantId::from_u128(2),
    name: "user2",
};

pub const USERS: [TestUser; 2] = [USER_1, USER_2];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TestContextVolatileStorage {
    Redis,
    Memory,
}

/// The [`TestContext`] provides access to redis & postgres for tests
pub struct TestContext {
    pub db_ctx: DatabaseContext,
    pub volatile: VolatileStorage,
    pub authz: Arc<Authz>,
    pub shutdown: Sender<()>,
}

impl TestContext {
    /// Creates a new [`TestContext`]
    pub async fn new(storage: TestContextVolatileStorage) -> Self {
        let _ = setup_logging();

        let db_ctx = DatabaseContext::new(true).await;

        let (shutdown, _) = tokio::sync::broadcast::channel(10);

        let enforcer = kustos::Authz::new(db_ctx.db.clone()).await.unwrap();

        let volatile = match storage {
            TestContextVolatileStorage::Redis => VolatileStorage::Right(redis::setup().await),
            TestContextVolatileStorage::Memory => {
                VolatileStorage::Left(VolatileStaticMemoryStorage)
            }
        };

        TestContext {
            db_ctx,
            volatile,
            authz: Arc::new(enforcer),
            shutdown,
        }
    }

    pub async fn default() -> Self {
        Self::new(TestContextVolatileStorage::Memory).await
    }
}

pub fn setup_logging() -> Result<(), Whatever> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()
        .whatever_context("Failed to setup logging utility")
}

/// Creates a new [`ModuleTester`] with two users
pub async fn setup_users<M: SignalingModule>(
    test_ctx: &TestContext,
    params: M::Params,
) -> (ModuleTester<M>, User, User) {
    let waiting_room = false;

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

    let room = test_ctx
        .db_ctx
        .create_test_room(ROOM_ID, user1.id, waiting_room)
        .await
        .unwrap();

    let mut module_tester = ModuleTester::new(
        test_ctx.db_ctx.db.clone(),
        test_ctx.authz.clone(),
        test_ctx.volatile.clone(),
        room,
    );

    // Join with user1
    module_tester
        .join_user(
            USER_1.participant_id,
            user1.clone(),
            Role::Moderator,
            &USER_1.name.parse().expect("valid display name"),
            params.clone(),
        )
        .await
        .unwrap();

    // Expect a JoinSuccess response
    if let WsMessageOutgoing::Control(ControlEvent::JoinSuccess(join_success)) = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap()
    {
        assert_eq!(join_success.id, USER_1.participant_id);
        assert_eq!(join_success.role, Role::Moderator);
        assert!(join_success.participants.is_empty());
    } else {
        panic!("Expected ParticipantJoined Event ")
    }

    // Join with user2
    module_tester
        .join_user(
            USER_2.participant_id,
            user2.clone(),
            Role::User,
            &USER_2.name.parse().expect("valid display name"),
            params.clone(),
        )
        .await
        .unwrap();

    // Expect a JoinSuccess on user2 websocket
    if let WsMessageOutgoing::Control(ControlEvent::JoinSuccess(join_success)) = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap()
    {
        assert_eq!(join_success.id, USER_2.participant_id);
        assert_eq!(join_success.role, Role::User);
        assert_eq!(join_success.participants.len(), 1);
    } else {
        panic!("Expected JoinSuccess message");
    }

    // Expect a ParticipantJoined event on user1 websocket
    if let WsMessageOutgoing::Control(ControlEvent::Joined(participant)) = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap()
    {
        assert_eq!(participant.id, USER_2.participant_id);
    } else {
        panic!("Expected ParticipantJoined Event ")
    }

    (module_tester, user1, user2)
}
