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
    web::{Data, Json, Path},
};
use anyhow::Context;
use opentalk_database::{Db, DbConnection};
use opentalk_db_storage::streaming_targets::{
    NewRoomStreamingTarget, RoomStreamingTargetRecord, UpdateRoomStreamingTarget,
};
use opentalk_types::{
    api::v1::{
        rooms::streaming_targets::{
            ChangeRoomStreamingTargetRequest, ChangeRoomStreamingTargetResponse,
            GetRoomStreamingTargetResponse, GetRoomStreamingTargetsResponse,
            PostRoomStreamingTargetRequest, PostRoomStreamingTargetResponse,
        },
        streaming_targets::StreamingTargetIdentifier,
    },
    common::streaming::{RoomStreamingTarget, StreamingTarget},
    core::RoomId,
};

/// API Endpoint *GET /rooms/{room_id}/streaming_targets*
///
/// Returns a JSON array of all streaming targets for the given room
#[get("/rooms/{room_id}/streaming_targets")]
pub async fn get_streaming_targets(
    db: Data<Db>,
    room_id: Path<RoomId>,
) -> DefaultApiResult<GetRoomStreamingTargetsResponse> {
    let mut conn = db.get_conn().await?;
    let room_id = room_id.into_inner();

    let room_streaming_targets = get_room_streaming_targets(&mut conn, room_id).await?;

    Ok(ApiResponse::new(GetRoomStreamingTargetsResponse(
        room_streaming_targets,
    )))
}

pub(super) async fn get_room_streaming_targets(
    conn: &mut DbConnection,
    room_id: RoomId,
) -> Result<Vec<RoomStreamingTarget>, ApiError> {
    let streaming_targets = RoomStreamingTargetRecord::get_all_for_room(conn, room_id).await?;

    let room_streaming_targets = streaming_targets
        .into_iter()
        .map(|room_streaming_target_record| {
            let room_streaming_target =
                create_room_streaming_target_from_record(room_streaming_target_record)?;

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

    let streaming_target_record = NewRoomStreamingTarget {
        service_id: streaming_target_clone.service_id,
        room_id,
        name: streaming_target_clone.name.clone(),
        streaming_url: streaming_target.streaming_url.map(|s| s.into()),
        streaming_key: streaming_target.streaming_key.map(|s| s.into()),
        public_url: streaming_target.public_url.map(|s| s.into()),
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
    path_params: Path<StreamingTargetIdentifier>,
) -> DefaultApiResult<GetRoomStreamingTargetResponse> {
    let mut conn = db.get_conn().await?;
    let StreamingTargetIdentifier {
        room_id,
        streaming_target_id,
    } = path_params.into_inner();

    let room_streaming_target_record =
        RoomStreamingTargetRecord::get(&mut conn, streaming_target_id, room_id).await?;

    let room_streaming_target =
        create_room_streaming_target_from_record(room_streaming_target_record)?;

    Ok(ApiResponse::new(GetRoomStreamingTargetResponse(
        room_streaming_target,
    )))
}

/// API Endpoint *PUT /rooms/{room_id}/streaming_targets/{streaming_target_id}*
///
/// Modifies and returns a single streaming target.
#[patch("/rooms/{room_id}/streaming_targets/{streaming_target_id}")]
pub async fn patch_streaming_target(
    db: Data<Db>,
    path_params: Path<StreamingTargetIdentifier>,
    streaming_target: Json<ChangeRoomStreamingTargetRequest>,
) -> DefaultApiResult<ChangeRoomStreamingTargetResponse> {
    let mut conn = db.get_conn().await?;
    let streaming_target = streaming_target.into_inner().0;

    if streaming_target.name.is_none()
        && streaming_target.streaming_url.is_none()
        && streaming_target.streaming_key.is_none()
        && streaming_target.public_url.is_none()
    {
        return Err(ApiError::bad_request());
    }

    let StreamingTargetIdentifier {
        room_id,
        streaming_target_id,
    } = path_params.into_inner();

    let room_streaming_target_record = UpdateRoomStreamingTarget {
        name: streaming_target.name,
        streaming_url: streaming_target.streaming_url.map(|s| s.map(|s| s.into())),
        streaming_key: streaming_target.streaming_key.map(|s| s.map(|s| s.into())),
        public_url: streaming_target.public_url.map(|s| s.map(|s| s.into())),
    };

    let room_streaming_target_record = room_streaming_target_record
        .apply(&mut conn, room_id, streaming_target_id)
        .await?;

    let room_streaming_target =
        create_room_streaming_target_from_record(room_streaming_target_record)?;

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
    path_params: Path<StreamingTargetIdentifier>,
) -> Result<NoContent, ApiError> {
    let mut conn = db.get_conn().await?;

    let StreamingTargetIdentifier {
        room_id,
        streaming_target_id,
    } = path_params.into_inner();

    RoomStreamingTargetRecord::delete_by_id(&mut conn, room_id, streaming_target_id).await?;

    Ok(NoContent)
}

fn create_room_streaming_target_from_record(
    room_streaming_target_record: RoomStreamingTargetRecord,
) -> Result<RoomStreamingTarget, ApiError> {
    let streaming_url = if let Some(streaming_url) = room_streaming_target_record.streaming_url {
        let streaming_url = streaming_url
            .parse()
            .context("invalid streaming endpoint url entry in db")?;
        Some(streaming_url)
    } else {
        None
    };

    let public_url = if let Some(public_url) = room_streaming_target_record.public_url {
        let public_url = public_url
            .parse()
            .context("invalid public url entry in db")?;
        Some(public_url)
    } else {
        None
    };

    let room_streaming_target = RoomStreamingTarget {
        id: room_streaming_target_record.id,
        streaming_target: StreamingTarget {
            service_id: room_streaming_target_record.service_id,
            name: room_streaming_target_record.name,
            streaming_url,
            streaming_key: room_streaming_target_record.streaming_key,
            public_url,
        },
    };
    Ok(room_streaming_target)
}
