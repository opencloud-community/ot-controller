// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::{ExpressionMethods, Identifiable, QueryDsl, Queryable};
use diesel_async::RunQueryDsl;
use opentalk_database::{DatabaseError, DbConnection, Result};
use opentalk_types::{
    common::streaming::{RoomStreamingTarget, StreamingTarget, StreamingTargetKind},
    core::{RoomId, StreamingKey, StreamingKind, StreamingTargetId},
    signaling::recording::{StreamKindSecret, StreamStatus, StreamTargetSecret},
};
use snafu::{Report, Snafu};

use crate::{rooms::Room, schema::room_streaming_targets};

#[derive(Debug, Snafu)]
pub enum StreamTargetConversionError {
    #[snafu(display("Parsing the url failed, because {target} is not a valid URL"))]
    WrongUrl { target: String },
}

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

impl TryFrom<RoomStreamingTargetRecord> for StreamTargetSecret {
    type Error = StreamTargetConversionError;

    fn try_from(value: RoomStreamingTargetRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            kind: StreamKindSecret::Livestream(match value.kind {
                StreamingKind::Custom => StreamingTargetKind::Custom {
                    streaming_endpoint: value.streaming_endpoint.parse().map_err(|_| {
                        StreamTargetConversionError::WrongUrl {
                            target: value.streaming_endpoint,
                        }
                    })?,
                    streaming_key: value.streaming_key,
                    public_url: value.public_url.parse().map_err(|_| {
                        StreamTargetConversionError::WrongUrl {
                            target: value.public_url,
                        }
                    })?,
                },
            }),
            status: StreamStatus::Inactive,
        })
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

pub async fn get_room_streaming_targets(
    conn: &mut DbConnection,
    room_id: RoomId,
) -> Result<Vec<RoomStreamingTarget>> {
    let streaming_targets = RoomStreamingTargetRecord::get_all_for_room(conn, room_id).await?;

    let room_streaming_targets = streaming_targets
        .into_iter()
        .map(|st| {
            let streaming_endpoint = st.streaming_endpoint.parse().map_err(|err| {
                log::warn!(
                    "Failed to parse streaming endpoint: {}",
                    Report::from_error(err)
                );
                DatabaseError::Custom {
                    message: "Inconsistent data".to_string(),
                }
            })?;
            let public_url = st.public_url.parse().map_err(|err| {
                log::warn!(
                    "Invalid public url entry in db: {}",
                    Report::from_error(err)
                );
                DatabaseError::Custom {
                    message: "Inconsistent data".to_string(),
                }
            })?;

            let room_streaming_target = RoomStreamingTarget {
                id: st.id,
                streaming_target: StreamingTarget {
                    name: st.name,
                    kind: StreamingTargetKind::Custom {
                        streaming_endpoint,
                        streaming_key: st.streaming_key,
                        public_url,
                    },
                },
            };

            Ok(room_streaming_target)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(room_streaming_targets)
}

pub async fn insert_room_streaming_target(
    conn: &mut DbConnection,
    room_id: RoomId,
    streaming_target: StreamingTarget,
) -> Result<RoomStreamingTarget> {
    let streaming_target_record = match &streaming_target.kind {
        StreamingTargetKind::Custom {
            streaming_endpoint,
            streaming_key,
            public_url,
        } => RoomStreamingTargetNew {
            room_id,
            name: streaming_target.name.clone(),
            kind: StreamingKind::Custom,
            streaming_endpoint: streaming_endpoint.clone().into(),
            streaming_key: streaming_key.clone().into(),
            public_url: public_url.clone().into(),
        },
    }
    .insert(conn)
    .await?;

    let room_streaming_target = RoomStreamingTarget {
        id: streaming_target_record.id,
        streaming_target,
    };
    Ok(room_streaming_target)
}
