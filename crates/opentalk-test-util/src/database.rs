// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use opentalk_database::Db;
use opentalk_db_storage::{
    groups::{get_or_create_groups_by_name, insert_user_into_groups},
    migrations::migrate_from_url,
    rooms::{NewRoom, Room},
    tariffs::Tariff,
    tenants::{OidcTenantId, get_or_create_tenant_by_oidc_id},
    users::{NewUser, User},
};
use opentalk_types_common::{
    rooms::RoomId,
    tariffs::TariffStatus,
    tenants::TenantId,
    users::{GroupName, UserId, UserTitle},
};
use snafu::{ResultExt, Whatever};

/// Contains the [`Db`] as well as information about the test database
pub struct DatabaseContext {
    pub base_url: String,
    pub db_name: String,
    pub db: Arc<Db>,
    /// DatabaseContext will DROP the database inside postgres when dropped
    pub drop_db_on_drop: bool,
}

impl DatabaseContext {
    /// Create a new [`DatabaseContext`]
    ///
    /// Uses the environment variable `POSTGRES_BASE_URL` to connect to postgres. Defaults to `postgres://postgres:password123@localhost:5432`
    /// when the environment variable is not set. The same goes for `DATABASE_NAME` where the default is `opentalk_test`.
    ///
    /// Once connected, the database with `DATABASE_NAME` gets dropped and re-created to guarantee a clean state, then the
    /// opentalk controller migration is applied.
    pub async fn new(drop_db_on_drop: bool) -> Self {
        let base_url = std::env::var("POSTGRES_BASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:password123@localhost:5432".to_owned());

        let db_name = std::env::var("DATABASE_NAME").unwrap_or_else(|_| "opentalk_test".to_owned());

        let postgres_url = format!("{base_url}/postgres");
        let mut conn = AsyncPgConnection::establish(&postgres_url)
            .await
            .expect("Cannot connect to postgres database.");

        // Drop the target database in case it already exists to guarantee a clean state
        drop_database(&mut conn, &db_name)
            .await
            .expect("Database initialization cleanup failed");

        // Create a new database for the test
        let query = diesel::sql_query(format!("CREATE DATABASE {db_name}"));
        query
            .execute(&mut conn)
            .await
            .unwrap_or_else(|_| panic!("Could not create database {db_name}"));

        let db_url = format!("{base_url}/{db_name}");

        migrate_from_url(&db_url)
            .await
            .expect("Unable to migrate database");

        let db_conn = Arc::new(Db::connect_url(&db_url, 5).unwrap());

        Self {
            base_url: base_url.to_string(),
            db_name: db_name.to_string(),
            db: db_conn,
            drop_db_on_drop,
        }
    }

    pub async fn create_test_user(&self, n: u32, groups: Vec<String>) -> Result<User, Whatever> {
        let mut conn = self
            .db
            .get_conn()
            .await
            .whatever_context("db connect failed")?;

        let tenant = get_or_create_tenant_by_oidc_id(
            &mut conn,
            &OidcTenantId::from("OpenTalkDefaultTenant".to_owned()),
        )
        .await
        .whatever_context("get/create tenant failed")?;
        let tariff = Tariff::get_by_name(&mut conn, "OpenTalkDefaultTariff")
            .await
            .unwrap();

        let user = NewUser {
            oidc_sub: format!("oidc_sub{n}"),
            email: format!("opentalk_test_user{n}@example.org"),
            title: UserTitle::new(),
            firstname: "test".into(),
            lastname: "tester".into(),
            avatar_url: Some("https://example.com/avatar/abcdef".into()),
            display_name: "test tester".parse().expect("valid display name"),
            language: "en".parse().expect("valid language"),
            phone: None,
            tenant_id: tenant.id,
            tariff_id: tariff.id,
            tariff_status: TariffStatus::Default,
        }
        .insert(&mut conn)
        .await
        .whatever_context("Insert user failed")?;

        let groups: Vec<(TenantId, GroupName)> = groups
            .into_iter()
            .map(|name| (tenant.id, GroupName::from(name)))
            .collect();
        let groups = get_or_create_groups_by_name(&mut conn, &groups)
            .await
            .whatever_context("create group failed")?;
        insert_user_into_groups(&mut conn, &user, &groups)
            .await
            .whatever_context("add user to group failed")?;

        Ok(user)
    }

    pub async fn create_test_room(
        &self,
        _room_id: RoomId,
        created_by: UserId,
        waiting_room: bool,
    ) -> Result<Room, Whatever> {
        let mut conn = self
            .db
            .get_conn()
            .await
            .whatever_context("db connect failed")?;

        let tenant =
            get_or_create_tenant_by_oidc_id(&mut conn, &OidcTenantId::from("default".to_owned()))
                .await
                .whatever_context("get or create tenant failed")?;

        let new_room = NewRoom {
            created_by,
            password: None,
            waiting_room,
            e2e_encryption: false,
            tenant_id: tenant.id,
        };

        let room = new_room
            .insert(&mut conn)
            .await
            .whatever_context("insert room failed")?;

        Ok(room)
    }
}

impl Drop for DatabaseContext {
    fn drop(&mut self) {
        if self.drop_db_on_drop {
            // Hack to avoid the missing "async drop"
            // Create a new runtime on a different thread, drop the database there and wait for the thread to complete.
            // The new thread is needed as tokio prevents creating a new runtime on a runtime thread.
            std::thread::scope(|s| {
                s.spawn(|| {
                    tokio::runtime::Runtime::new()
                        .unwrap()
                        .block_on(async move {
                            let postgres_url = format!("{}/postgres", self.base_url);
                            let db_name = self.db_name.clone();

                            let mut conn = AsyncPgConnection::establish(&postgres_url)
                                .await
                                .expect("Cannot connect to postgres database.");

                            drop_database(&mut conn, &db_name).await.unwrap();
                        })
                });
            });
        }
    }
}

/// Disconnect all users from the database with `db_name` and drop it.
async fn drop_database(conn: &mut AsyncPgConnection, db_name: &str) -> Result<(), Whatever> {
    let query = diesel::sql_query(format!("DROP DATABASE IF EXISTS {db_name} WITH (FORCE)"));
    query
        .execute(conn)
        .await
        .with_whatever_context(|_| format!("Couldn't drop database {db_name}"))?;

    Ok(())
}
