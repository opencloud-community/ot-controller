// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use database::{DbConnection, Result};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use types::{
    common::shared_folder::{SharedFolder, SharedFolderAccess},
    core::{EventId, RoomId},
};

use super::Event;

use crate::schema::{event_shared_folders, events};

#[derive(Insertable)]
#[diesel(table_name = event_shared_folders)]
pub struct NewEventSharedFolder {
    pub event_id: EventId,
    pub path: String,
    pub write_share_id: String,
    pub write_url: String,
    pub write_password: String,
    pub read_share_id: String,
    pub read_url: String,
    pub read_password: String,
}

impl NewEventSharedFolder {
    /// Tries to insert the EventSharedFolder into the database
    ///
    /// When yielding a unique constraint violation, None is returned.
    #[tracing::instrument(err, skip_all)]
    pub async fn try_insert(self, conn: &mut DbConnection) -> Result<Option<EventSharedFolder>> {
        let query = self.insert_into(event_shared_folders::table);

        let result = query.get_result(conn).await;

        match result {
            Ok(event_shared_folders) => Ok(Some(event_shared_folders)),
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                ..,
            )) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Associations, Identifiable, Queryable)]
#[diesel(table_name = event_shared_folders)]
#[diesel(primary_key(event_id))]
#[diesel(belongs_to(Event))]
pub struct EventSharedFolder {
    pub event_id: EventId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub path: String,
    pub write_share_id: String,
    pub write_url: String,
    pub write_password: String,
    pub read_share_id: String,
    pub read_url: String,
    pub read_password: String,
}

impl EventSharedFolder {
    #[tracing::instrument(err, skip_all)]
    pub async fn get_for_event(
        conn: &mut DbConnection,
        event_id: EventId,
    ) -> Result<Option<EventSharedFolder>> {
        let shared_folder = event_shared_folders::table
            .filter(event_shared_folders::event_id.eq(event_id))
            .get_result(conn)
            .await
            .optional()?;

        Ok(shared_folder)
    }

    /// Returns all [`EventSharedFolder`]s in the given [`RoomId`].
    #[tracing::instrument(err, skip_all)]
    pub async fn get_all_for_room(
        conn: &mut DbConnection,
        room_id: RoomId,
    ) -> Result<Vec<EventSharedFolder>> {
        let query = event_shared_folders::table
            .inner_join(events::table)
            .filter(events::room.eq(room_id))
            .select(event_shared_folders::all_columns);

        let events = query.load(conn).await?;

        Ok(events)
    }

    /// Delete a shared folder using the given event id
    #[tracing::instrument(err, skip_all)]
    pub async fn delete_by_event_id(conn: &mut DbConnection, event_id: EventId) -> Result<()> {
        let query = diesel::delete(
            event_shared_folders::table.filter(event_shared_folders::event_id.eq(event_id)),
        );

        query.execute(conn).await?;

        Ok(())
    }

    /// Delete shared folders using the given event ids
    #[tracing::instrument(err, skip_all)]
    pub async fn delete_by_event_ids(conn: &mut DbConnection, event_ids: &[EventId]) -> Result<()> {
        let query = diesel::delete(
            event_shared_folders::table.filter(event_shared_folders::event_id.eq_any(event_ids)),
        );

        query.execute(conn).await?;

        Ok(())
    }

    /// Delete the shared folder for an event
    #[tracing::instrument(err, skip_all)]
    pub async fn delete(self, conn: &mut DbConnection) -> Result<()> {
        Self::delete_by_event_id(conn, self.event_id).await
    }
}

impl From<EventSharedFolder> for SharedFolder {
    fn from(
        EventSharedFolder {
            write_password,
            write_url,
            read_password,
            read_url,
            ..
        }: EventSharedFolder,
    ) -> Self {
        SharedFolder {
            read: SharedFolderAccess {
                url: read_url,
                password: read_password,
            },
            read_write: Some(SharedFolderAccess {
                url: write_url,
                password: write_password,
            }),
        }
    }
}
