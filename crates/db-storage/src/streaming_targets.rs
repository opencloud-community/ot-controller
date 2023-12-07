// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::schema::room_streaming_targets;
use database::{DbConnection, Result};
use diesel::{ExpressionMethods, Identifiable, QueryDsl, Queryable};
use diesel_async::RunQueryDsl;
use types::core::{RoomId, StreamingKey, StreamingKind, StreamingTargetId};

use crate::rooms::Room;

#[derive(Debug, Queryable, Identifiable, Associations, Insertable)]
#[diesel(belongs_to(Room, foreign_key = room_id))]
#[diesel(table_name = room_streaming_targets)]
pub struct RoomStreamingTargetRecord {
    pub id: StreamingTargetId,
    pub room_id: RoomId,
    pub name: String,
    pub kind: StreamingKind,
    pub streaming_endpoint: String,
    pub streaming_key: StreamingKey,
    pub public_url: String,
}

impl RoomStreamingTargetRecord {
    /// Retrieve a single streaming target
    #[tracing::instrument(err, skip_all)]
    pub async fn get(
        conn: &mut DbConnection,
        streaming_target_id: StreamingTargetId,
        room_id: RoomId,
    ) -> Result<RoomStreamingTargetRecord> {
        let streaming_target = room_streaming_targets::table
            .filter(room_streaming_targets::id.eq(streaming_target_id))
            .filter(room_streaming_targets::room_id.eq(room_id))
            .first(conn)
            .await?;

        Ok(streaming_target)
    }

    /// Retrieve all streaming targets
    #[tracing::instrument(err, skip_all)]
    pub async fn get_all_for_room(
        conn: &mut DbConnection,
        room_id: RoomId,
    ) -> Result<Vec<RoomStreamingTargetRecord>> {
        let streaming_targets = room_streaming_targets::table
            .filter(room_streaming_targets::room_id.eq(room_id))
            .load(conn)
            .await?;

        Ok(streaming_targets)
    }

    /// Delete a streaming target using the given room & streaming target id
    #[tracing::instrument(err, skip_all)]
    pub async fn delete_by_id(
        conn: &mut DbConnection,
        room_id: RoomId,
        streaming_target_id: StreamingTargetId,
    ) -> Result<()> {
        let _ = diesel::delete(
            room_streaming_targets::table
                .filter(room_streaming_targets::id.eq(streaming_target_id))
                .filter(room_streaming_targets::room_id.eq(room_id)),
        )
        .execute(conn)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Associations, Insertable)]
#[diesel(belongs_to(Room, foreign_key = room_id))]
#[diesel(table_name = room_streaming_targets)]
pub struct RoomStreamingTargetNew {
    pub room_id: RoomId,
    pub name: String,
    pub kind: StreamingKind,
    pub streaming_endpoint: String,
    pub streaming_key: String,
    pub public_url: String,
}

impl RoomStreamingTargetNew {
    #[tracing::instrument(err, skip_all)]
    pub async fn insert(self, conn: &mut DbConnection) -> Result<RoomStreamingTargetRecord> {
        let query = diesel::insert_into(room_streaming_targets::table).values(self);

        let room_streaming_target: RoomStreamingTargetRecord = query.get_result(conn).await?;

        Ok(room_streaming_target)
    }
}

/// Diesel streaming target struct
///
/// Represents a changeset of in invite
#[derive(Debug, AsChangeset)]
#[diesel(table_name = room_streaming_targets)]
pub struct UpdateRoomStreamingTarget {
    pub name: Option<String>,
    pub kind: Option<StreamingKind>,
    pub streaming_endpoint: Option<String>,
    pub streaming_key: Option<String>,
    pub public_url: Option<String>,
}

impl UpdateRoomStreamingTarget {
    #[tracing::instrument(err, skip_all)]
    pub async fn apply(
        self,
        conn: &mut DbConnection,
        room_id: RoomId,
        streaming_target_id: StreamingTargetId,
    ) -> Result<RoomStreamingTargetRecord> {
        let query = diesel::update(room_streaming_targets::table)
            .filter(room_streaming_targets::id.eq(streaming_target_id))
            .filter(room_streaming_targets::room_id.eq(room_id))
            .set(self)
            .returning(room_streaming_targets::all_columns);

        let invite = query.get_result(conn).await?;

        Ok(invite)
    }
}
