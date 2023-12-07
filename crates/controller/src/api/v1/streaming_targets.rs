// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains invite related REST endpoints.
use super::{
    response::{ApiError, NoContent},
    DefaultApiResult,
};
use crate::api::v1::ApiResponse;
use actix_web::{
    delete, get, patch, post,
    web::{Data, Json, Path, Query},
};
use anyhow::Context;
use database::{Db, DbConnection};
use db_storage::streaming_targets::{
    RoomStreamingTargetNew, RoomStreamingTargetRecord, UpdateRoomStreamingTarget,
};
use types::{
    api::v1::{
        pagination::PagePaginationQuery,
        rooms::streaming_targets::{
            ChangeRoomStreamingTargetRequest, ChangeRoomStreamingTargetResponse,
            GetRoomStreamingTargetResponse, GetRoomStreamingTargetsResponse,
            PostRoomStreamingTargetRequest, PostRoomStreamingTargetResponse,
        },
        streaming_targets::{RoomAndStreamingTargetId, UpdateStreamingTargetKind},
    },
    common::streaming::{RoomStreamingTarget, StreamingTarget, StreamingTargetKind},
    core::RoomId,
    core::StreamingKind,
};

/// API Endpoint *GET /rooms/{room_id}/streaming_targets*
///
/// Returns a JSON array of all streaming targets for the given room
#[get("/rooms/{room_id}/streaming_targets")]
pub async fn get_streaming_targets(
    db: Data<Db>,
    room_id: Path<RoomId>,
    pagination: Query<PagePaginationQuery>,
) -> DefaultApiResult<GetRoomStreamingTargetsResponse> {
    let mut conn = db.get_conn().await?;
    let room_id = room_id.into_inner();
    let PagePaginationQuery { per_page, page } = pagination.into_inner();

    let room_streaming_targets = get_room_streaming_targets(&mut conn, room_id).await?;

    let len = room_streaming_targets.len();

    Ok(
        ApiResponse::new(GetRoomStreamingTargetsResponse(room_streaming_targets))
            .with_page_pagination(per_page, page, len as i64),
    )
}

pub(super) async fn get_room_streaming_targets(
    conn: &mut DbConnection,
    room_id: RoomId,
) -> Result<Vec<RoomStreamingTarget>, ApiError> {
    let streaming_targets = RoomStreamingTargetRecord::get_all_for_room(conn, room_id).await?;

    let room_streaming_targets = streaming_targets
        .into_iter()
        .map(|st| {
            let streaming_endpoint = st
                .streaming_endpoint
                .parse()
                .context("invalid streaming endpoint url entry in db")?;
            let public_url = st
                .public_url
                .parse()
                .context("invalid public url entry in db")?;

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
        .collect::<Result<Vec<_>, ApiError>>()?;
    Ok(room_streaming_targets)
}

/// API Endpoint *POST /rooms/{room_id}/streaming_targets*
///
/// Creates a new streaming target for the given room
#[post("/rooms/{room_id}/streaming_targets")]
pub async fn post_streaming_target(
    db: Data<Db>,
    room_id: Path<RoomId>,
    data: Json<PostRoomStreamingTargetRequest>,
) -> DefaultApiResult<PostRoomStreamingTargetResponse> {
    let mut conn = db.get_conn().await?;
    let room_id = room_id.into_inner();
    let streaming_target = data.into_inner().0;

    let room_streaming_target =
        insert_room_streaming_target(&mut conn, room_id, streaming_target).await?;

    Ok(ApiResponse::new(PostRoomStreamingTargetResponse(
        room_streaming_target,
    )))
}

pub(super) async fn insert_room_streaming_target(
    conn: &mut DbConnection,
    room_id: RoomId,
    streaming_target: StreamingTarget,
) -> Result<RoomStreamingTarget, ApiError> {
    let streaming_target_clone = streaming_target.clone();

    let streaming_target_record = match streaming_target.kind {
        StreamingTargetKind::Custom {
            streaming_endpoint,
            streaming_key,
            public_url,
        } => RoomStreamingTargetNew {
            room_id,
            name: streaming_target_clone.name.clone(),
            kind: StreamingKind::Custom,
            streaming_endpoint: streaming_endpoint.into(),
            streaming_key: streaming_key.into(),
            public_url: public_url.into(),
        },
    }
    .insert(conn)
    .await?;

    let room_streaming_target = RoomStreamingTarget {
        id: streaming_target_record.id,
        streaming_target: streaming_target_clone,
    };
    Ok(room_streaming_target)
}

/// API Endpoint *GET /rooms/{room_id}/streaming_targets/{streaming_target_id}*
///
/// Returns a single streaming target.
/// Returns 401 Not Found when the user has no access.
#[get("/rooms/{room_id}/streaming_targets/{streaming_target_id}")]
pub async fn get_streaming_target(
    db: Data<Db>,
    path_params: Path<RoomAndStreamingTargetId>,
) -> DefaultApiResult<GetRoomStreamingTargetResponse> {
    let mut conn = db.get_conn().await?;
    let RoomAndStreamingTargetId {
        room_id,
        streaming_target_id,
    } = path_params.into_inner();

    let room_streaming_target =
        RoomStreamingTargetRecord::get(&mut conn, streaming_target_id, room_id).await?;

    Ok(ApiResponse::new(GetRoomStreamingTargetResponse(
        RoomStreamingTarget {
            id: room_streaming_target.id,
            streaming_target: StreamingTarget {
                name: room_streaming_target.name,
                kind: StreamingTargetKind::Custom {
                    streaming_endpoint: room_streaming_target
                        .streaming_endpoint
                        .parse()
                        .context("invalid streaming endpoint url entry in db")?,
                    streaming_key: room_streaming_target.streaming_key,
                    public_url: room_streaming_target
                        .public_url
                        .parse()
                        .context("invalid public url entry in db")?,
                },
            },
        },
    )))
}

/// API Endpoint *PUT /rooms/{room_id}/streaming_targets/{streaming_target_id}*
///
/// Modifies and returns a single streaming target.
#[patch("/rooms/{room_id}/streaming_targets/{streaming_target_id}")]
pub async fn patch_streaming_target(
    db: Data<Db>,
    path_params: Path<RoomAndStreamingTargetId>,
    update_streaming_target: Json<ChangeRoomStreamingTargetRequest>,
) -> DefaultApiResult<ChangeRoomStreamingTargetResponse> {
    let mut conn = db.get_conn().await?;
    let update_streaming_target = update_streaming_target.into_inner().0;

    if update_streaming_target.name.is_none() && update_streaming_target.kind.is_none() {
        return Err(ApiError::bad_request());
    }

    let RoomAndStreamingTargetId {
        room_id,
        streaming_target_id,
    } = path_params.into_inner();

    let (kind, streaming_endpoint, streaming_key, public_url) = match update_streaming_target.kind {
        Some(kind) => match kind {
            UpdateStreamingTargetKind::Custom {
                streaming_endpoint,
                streaming_key,
                public_url,
            } => (
                Some(StreamingKind::Custom),
                streaming_endpoint.map(|s| s.into()),
                streaming_key.map(|s| s.into()),
                public_url.map(|s| s.into()),
            ),
        },
        None => (None, None, None, None),
    };

    let room_streaming_target_table = UpdateRoomStreamingTarget {
        name: update_streaming_target.name,
        kind,
        streaming_endpoint,
        streaming_key,
        public_url,
    };

    let room_streaming_target_table = room_streaming_target_table
        .apply(&mut conn, room_id, streaming_target_id)
        .await?;

    let kind = match room_streaming_target_table.kind {
        StreamingKind::Custom => StreamingTargetKind::Custom {
            streaming_endpoint: room_streaming_target_table
                .streaming_endpoint
                .parse()
                .context("invalid streaming endpoint url entry in db")?,
            streaming_key: room_streaming_target_table.streaming_key,
            public_url: room_streaming_target_table
                .public_url
                .parse()
                .context("invalid public url entry in db")?,
        },
    };

    let room_streaming_target = RoomStreamingTarget {
        id: room_streaming_target_table.id,
        streaming_target: StreamingTarget {
            name: room_streaming_target_table.name,
            kind,
        },
    };

    Ok(ApiResponse::new(ChangeRoomStreamingTargetResponse(
        room_streaming_target,
    )))
}

/// API Endpoint *DELETE /rooms/{room_id}/streaming_targets/{streaming_target_id}*
///
/// Deletes a single streaming target.
/// Returns 204 No Content
#[delete("/rooms/{room_id}/streaming_targets/{streaming_target_id}")]
pub async fn delete_streaming_target(
    db: Data<Db>,
    path_params: Path<RoomAndStreamingTargetId>,
) -> Result<NoContent, ApiError> {
    let mut conn = db.get_conn().await?;

    let RoomAndStreamingTargetId {
        room_id,
        streaming_target_id,
    } = path_params.into_inner();

    RoomStreamingTargetRecord::delete_by_id(&mut conn, room_id, streaming_target_id).await?;

    Ok(NoContent)
}
