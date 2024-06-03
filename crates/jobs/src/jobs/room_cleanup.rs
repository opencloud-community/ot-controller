// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::HashSet, sync::Arc};

use async_trait::async_trait;
use kustos::Authz;
use log::Log;
use opentalk_controller_settings::Settings;
use opentalk_database::{Db, DbConnection};
use opentalk_db_storage::rooms::Room;
use opentalk_log::{debug, info};
use opentalk_signaling_core::{ExchangeHandle, ObjectStorage};
use opentalk_types::core::RoomId;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::{
    error::{ParameterLoadingSnafu, ParameterSerializingSnafu},
    events::delete_orphaned_rooms,
    Error, Job, JobParameters,
};

#[derive(Debug)]
pub struct RoomCleanup;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RoomCleanupParameters {
    #[serde(default)]
    fail_on_shared_folder_deletion_error: bool,
}

impl JobParameters for RoomCleanupParameters {
    fn try_from_json(json: serde_json::Value) -> Result<Self, Error> {
        serde_json::from_value(json).context(ParameterLoadingSnafu)
    }

    fn to_json(&self) -> Result<serde_json::Value, Error> {
        serde_json::to_value(self).context(ParameterSerializingSnafu)
    }
}

#[async_trait]
impl Job for RoomCleanup {
    type Parameters = RoomCleanupParameters;

    async fn execute(
        logger: &dyn Log,
        db: Arc<Db>,
        exchange_handle: ExchangeHandle,
        settings: &Settings,
        parameters: Self::Parameters,
    ) -> Result<(), Error> {
        info!(log: logger, "Starting orphaned rooms cleanup job");
        debug!(log: logger, "Job parameters: {parameters:?}");

        let mut conn = db.get_conn().await?;

        let authz = Authz::new(db.clone()).await?;

        let object_storage = ObjectStorage::new(&settings.minio).await?;

        let orphaned_rooms = find_orphaned_rooms(&mut conn).await?;

        if orphaned_rooms.is_empty() {
            info!(log: logger, "No orphaned rooms found. Job finished!");
            return Ok(());
        }

        delete_orphaned_rooms(
            logger,
            &mut conn,
            &authz,
            exchange_handle,
            settings,
            &object_storage,
            orphaned_rooms,
            parameters.fail_on_shared_folder_deletion_error,
        )
        .await?;

        Ok(())
    }
}

async fn find_orphaned_rooms(conn: &mut DbConnection) -> Result<HashSet<RoomId>, Error> {
    let rooms = Room::get_all_orphaned_ids(conn).await?;

    Ok(HashSet::from_iter(rooms.into_iter()))
}

#[cfg(test)]
mod test {
    use opentalk_database::DbConnection;
    use opentalk_db_storage::{
        events::{Event, NewEvent},
        rooms::{NewRoom, Room},
        users::User,
    };
    use opentalk_test_util::database::DatabaseContext;

    /// Test to fill the database with events and independent rooms. Is ignored by the CI
    ///
    /// The created data is persistent and saved in `opentalk_test`. The target database can be overwritten with the
    /// `DATABASE_NAME` environment variable. For more configuration options, see
    /// [`opentalk_test_util::database::DatabaseContext`].
    ///
    /// Run with:
    /// `cargo test --package opentalk-jobs  -- --show-output --exact jobs::room_cleanup::test::fill_db_with_test_data --nocapture --ignored`
    #[ignore]
    #[actix_rt::test]
    async fn fill_db_with_test_data() {
        let db_ctx = DatabaseContext::new(false).await;

        create_events_and_independent_rooms(&db_ctx, 50, 100).await;
    }

    async fn create_events_and_independent_rooms(
        db_ctx: &DatabaseContext,
        event_count: u64,
        independent_room_count: u64,
    ) {
        let user = db_ctx.create_test_user(1, vec![]).await.unwrap();

        let mut conn = db_ctx.db.get_conn().await.unwrap();

        for _ in 0..event_count {
            create_generic_test_event(&mut conn, &user).await;
        }

        for _ in 0..independent_room_count {
            create_generic_test_room(&mut conn, &user).await;
        }
    }

    async fn create_generic_test_room(conn: &mut DbConnection, user: &User) -> Room {
        let new_room = NewRoom {
            created_by: user.id,
            password: None,
            waiting_room: false,
            tenant_id: user.tenant_id,
        };

        new_room.insert(conn).await.unwrap()
    }

    async fn create_generic_test_event(conn: &mut DbConnection, user: &User) -> Event {
        let room = create_generic_test_room(conn, user).await;

        let new_event = NewEvent {
            title: "TestEvent".into(),
            description: "A normal event, created by a test".into(),
            room: room.id,
            created_by: user.id,
            updated_by: user.id,
            is_time_independent: true,
            is_all_day: None,
            starts_at: None,
            starts_at_tz: None,
            ends_at: None,
            ends_at_tz: None,
            duration_secs: None,
            is_recurring: None,
            recurrence_pattern: None,
            is_adhoc: true,
            tenant_id: user.tenant_id,
            show_meeting_details: true,
        };

        new_event.insert(conn).await.unwrap()
    }
}
