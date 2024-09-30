// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use kustos::prelude::*;
use opentalk_database::Db;

fn init_log() {
    let _ = env_logger::try_init();
}

/// Grant access, revoke access and grant again (two resources).
///
/// A user should have access after access rights where granted a second time.
///
/// NOTE: Since the database is not cleaned up between tests runs and all tests
///       share the same database, we need to manually clean up the database
#[tokio::test]
async fn grant_revoke_grant_two_resources() -> Result<(), Box<dyn std::error::Error>> {
    init_log();

    let url = std::env::var("KUSTOS_TESTS_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password123@localhost:5432/kustos".to_string());
    let db = Arc::new(Db::connect_url(&url, 10).unwrap());

    let authz = Authz::new(db).await.unwrap();

    let invitee = uuid::Uuid::from_u128(0xfa4e2ab5_2223_429f_9201_5a1e3b889714);

    let event = uuid::Uuid::from_u128(0x5f81c3cd_16b3_4d6b_9853_b3bf926401b1);
    let room = uuid::Uuid::from_u128(0xddab0058_9c8b_44c9_a704_6d50c3a496c6);

    let event_resource = format!("/events/{}", event);
    let room_resource = format!("/rooms/{}", room);

    let policies = PoliciesBuilder::new()
        .grant_user_access(invitee)
        .add_resource(event_resource.as_str(), [AccessMethod::POST])
        .add_resource(room_resource.as_str(), [AccessMethod::GET])
        .finish();
    authz.add_policies(policies.clone()).await?;

    let res = authz
        .check_user(invitee, event_resource.as_str(), AccessMethod::POST)
        .await?;
    assert!(res, "Invitee should have event access");
    let res = authz
        .check_user(invitee, room_resource.as_str(), AccessMethod::GET)
        .await?;
    assert!(res, "Invitee should have room access");

    authz.remove_policies(policies.clone()).await?;

    let res = authz
        .check_user(invitee, event_resource.as_str(), AccessMethod::POST)
        .await?;
    assert!(!res, "Invitee access should be revoked");

    authz.add_policies(policies.clone()).await?;

    let res = authz
        .check_user(invitee, event_resource.as_str(), AccessMethod::POST)
        .await?;
    assert!(
        res,
        "Invitee should have event access after granting it again"
    );
    let res = authz
        .check_user(invitee, room_resource.as_str(), AccessMethod::GET)
        .await?;
    assert!(
        res,
        "Invitee should have room access after granting it again"
    );

    // clean up since the database is used for all tests.
    authz.remove_policies(policies.clone()).await?;

    Ok(())
}
