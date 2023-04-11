// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::common::make_user;
use opentalk_db_storage::users::User;
use pretty_assertions::assert_eq;
use serial_test::serial;

mod common;

#[tokio::test]
#[serial]
async fn test() {
    let db_ctx = test_util::database::DatabaseContext::new(true).await;
    let mut conn = db_ctx.db.get_conn().await.unwrap();

    // generate some random users with some made up names
    let tenant_id = make_user(&mut conn, "Aileen", "Strange", "Spectre")
        .await
        .tenant_id;
    make_user(&mut conn, "Laura", "Rutherford", "Jakiro").await;
    make_user(&mut conn, "Cheryl", "Lazarus", "Kaolin").await;

    let users = User::find(&mut conn, tenant_id, "La").await.unwrap();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].firstname, "Cheryl");
    assert_eq!(users[1].firstname, "Laura");

    let users = User::find(&mut conn, tenant_id, "Ru").await.unwrap();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].firstname, "Laura");
    assert_eq!(users[1].firstname, "Cheryl");

    // Try the levenshtein/soundex matching with worse input each time
    let users = User::find(&mut conn, tenant_id, "Cheril Lazarus")
        .await
        .unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].firstname, "Cheryl");

    let users = User::find(&mut conn, tenant_id, "Cheril Lasarus")
        .await
        .unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].firstname, "Cheryl");

    let users = User::find(&mut conn, tenant_id, "Cherill Lasarus")
        .await
        .unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].firstname, "Cheryl");

    let users = User::find(&mut conn, tenant_id, "Cherill Lasaruz")
        .await
        .unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].firstname, "Cheryl");

    let users = User::find(&mut conn, tenant_id, "Spectre").await.unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].firstname, "Aileen");

    let users = User::find(&mut conn, tenant_id, "Spektre").await.unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].firstname, "Aileen");

    let users = User::find(&mut conn, tenant_id, "Schpecktre")
        .await
        .unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].firstname, "Aileen");
}
