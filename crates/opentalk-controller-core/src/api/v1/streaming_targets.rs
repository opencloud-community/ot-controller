// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains invite related REST endpoints.
use actix_web::{
    delete, get, patch, post,
    web::{Data, Json, Path, Query, ReqData},
};
use opentalk_database::Db;
use opentalk_db_storage::{
    rooms::Room,
    streaming_targets::{
        get_room_streaming_targets, insert_room_streaming_target, RoomStreamingTargetRecord,
        UpdateRoomStreamingTarget,
    },
    tenants::Tenant,
    users::User,
};
use opentalk_keycloak_admin::KeycloakAdminClient;
use opentalk_types::api::{
    error::ApiError,
    v1::{
        pagination::PagePaginationQuery,
        rooms::streaming_targets::{
            ChangeRoomStreamingTargetRequest, ChangeRoomStreamingTargetResponse,
            GetRoomStreamingTargetResponse, GetRoomStreamingTargetsResponse,
            PostRoomStreamingTargetRequest, PostRoomStreamingTargetResponse,
        },
        streaming_targets::{RoomAndStreamingTargetId, UpdateStreamingTargetKind},
    },
};
use opentalk_types_api_v1::events::StreamingTargetOptionsQuery;
use opentalk_types_common::{
    rooms::RoomId,
    streaming::{
        RoomStreamingTarget, RoomStreamingTargetResource, StreamingKind, StreamingTarget,
        StreamingTargetKind, StreamingTargetKindResource, StreamingTargetResource,
    },
};
use snafu::Report;

use super::{response::NoContent, DefaultApiResult};
use crate::{
    api::{
        responses::{Forbidden, InternalServerError, NotFound, Unauthorized},
        v1::{events::notify_event_invitees_by_room_about_update, ApiResponse},
    },
    services::MailService,
    settings::SharedSettingsActix,
};

/// List the streaming targets of a room
///
/// Returns the streaming targets available for a room
#[utoipa::path(
    params(
        PagePaginationQuery,
        ("room_id" = RoomId, description = "The id of the room"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "List of streaming targets successfully returned",
            body = GetRoomStreamingTargetsResponse,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::FORBIDDEN,
            response = Forbidden,
        ),
        (
            status = StatusCode::NOT_FOUND,
            response = NotFound,
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[get("/rooms/{room_id}/streaming_targets")]
pub async fn get_streaming_targets(
    db: Data<Db>,
    current_user: ReqData<User>,
    room_id: Path<RoomId>,
    pagination: Query<PagePaginationQuery>,
) -> DefaultApiResult<GetRoomStreamingTargetsResponse> {
    let mut conn = db.get_conn().await?;
    let current_user_id = current_user.into_inner().id;
    let room_id = room_id.into_inner();
    let PagePaginationQuery { per_page, page } = pagination.into_inner();

    let room_streaming_targets = get_room_streaming_targets(&mut conn, room_id).await?;
    let len = room_streaming_targets.len();

    let room = Room::get(&mut conn, room_id).await?;
    let with_streaming_key = room.created_by == current_user_id;
    let room_streaming_target_resources = room_streaming_targets
        .into_iter()
        .map(|rst| build_resource(rst, with_streaming_key))
        .collect();

    Ok(ApiResponse::new(GetRoomStreamingTargetsResponse(
        room_streaming_target_resources,
    ))
    .with_page_pagination(per_page, page, len as i64))
}

/// Create a new streaming target
///
/// Creates a new streaming target for the given room
#[utoipa::path(
    params(
        StreamingTargetOptionsQuery,
        ("room_id" = RoomId, description = "The id of the room"),
    ),
    request_body = PostRoomStreamingTargetRequest,
    responses(
        (
            status = StatusCode::OK,
            description = "Successfully create a new streaming target",
            body = PostRoomStreamingTargetResponse,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::FORBIDDEN,
            response = Forbidden,
        ),
        (
            status = StatusCode::NOT_FOUND,
            response = NotFound,
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[post("/rooms/{room_id}/streaming_targets")]
#[allow(clippy::too_many_arguments)]
pub async fn post_streaming_target(
    settings: SharedSettingsActix,
    db: Data<Db>,
    kc_admin_client: Data<KeycloakAdminClient>,
    mail_service: Data<MailService>,
    current_tenant: ReqData<Tenant>,
    current_user: ReqData<User>,
    room_id: Path<RoomId>,
    query: Query<StreamingTargetOptionsQuery>,
    data: Json<PostRoomStreamingTargetRequest>,
) -> DefaultApiResult<PostRoomStreamingTargetResponse> {
    let settings = settings.load_full();
    let mail_service = mail_service.into_inner();
    let current_tenant = current_tenant.into_inner();
    let current_user = current_user.into_inner();
    let room_id = room_id.into_inner();
    let query = query.into_inner();
    let streaming_target = data.into_inner().0;

    let send_email_notification = !query.suppress_email_notification;

    let mut conn = db.get_conn().await?;

    let room_streaming_target =
        insert_room_streaming_target(&mut conn, room_id, streaming_target).await?;

    if send_email_notification {
        notify_event_invitees_by_room_about_update(
            &kc_admin_client,
            settings,
            mail_service,
            current_tenant,
            current_user,
            &mut conn,
            room_id,
        )
        .await?;
    }

    Ok(ApiResponse::new(PostRoomStreamingTargetResponse(
        room_streaming_target,
    )))
}

/// Get a streaming target
///
/// Returns a single streaming target for a specific room.
#[utoipa::path(
    params(RoomAndStreamingTargetId),
    responses(
        (
            status = StatusCode::OK,
            description = "The streaming target has been successfully returned",
            body = GetRoomStreamingTargetResponse,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::FORBIDDEN,
            response = Forbidden,
        ),
        (
            status = StatusCode::NOT_FOUND,
            response = NotFound,
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[get("/rooms/{room_id}/streaming_targets/{streaming_target_id}")]
pub async fn get_streaming_target(
    db: Data<Db>,
    current_user: ReqData<User>,
    path_params: Path<RoomAndStreamingTargetId>,
) -> DefaultApiResult<GetRoomStreamingTargetResponse> {
    let mut conn = db.get_conn().await?;
    let current_user_id = current_user.into_inner().id;
    let RoomAndStreamingTargetId {
        room_id,
        streaming_target_id,
    } = path_params.into_inner();

    let room_streaming_target =
        RoomStreamingTargetRecord::get(&mut conn, streaming_target_id, room_id).await?;

    let room_streaming_target = RoomStreamingTarget {
        id: room_streaming_target.id,
        streaming_target: StreamingTarget {
            name: room_streaming_target.name,
            kind: StreamingTargetKind::Custom {
                streaming_endpoint: room_streaming_target.streaming_endpoint.parse().map_err(
                    |e| {
                        log::warn!(
                            "Invalid streaming endpoint url entry in db: {}",
                            Report::from_error(e)
                        );
                        ApiError::internal()
                    },
                )?,
                streaming_key: room_streaming_target.streaming_key,
                public_url: room_streaming_target.public_url.parse().map_err(|e| {
                    log::warn!(
                        "Invalid streaming endpoint url entry in db: {}",
                        Report::from_error(e)
                    );
                    ApiError::internal()
                })?,
            },
        },
    };

    let room = Room::get(&mut conn, room_id).await?;
    let with_streaming_key = room.created_by == current_user_id;
    let room_streaming_target_resource = build_resource(room_streaming_target, with_streaming_key);

    Ok(ApiResponse::new(GetRoomStreamingTargetResponse(
        room_streaming_target_resource,
    )))
}

/// Update a room streaming target
///
/// Modifies and returns a single streaming target.
#[utoipa::path(
    params(RoomAndStreamingTargetId),
    request_body = ChangeRoomStreamingTargetRequest,
    responses(
        (
            status = StatusCode::OK,
            description = "Streaming target was successfully updated",
            body = ChangeRoomStreamingTargetResponse
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = r"Could not modify the specified streaming target due to wrong
                syntax or bad values",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = HttpStatus::FORBIDDEN,
            response = Forbidden,
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[patch("/rooms/{room_id}/streaming_targets/{streaming_target_id}")]
#[allow(clippy::too_many_arguments)]
pub async fn patch_streaming_target(
    settings: SharedSettingsActix,
    db: Data<Db>,
    kc_admin_client: Data<KeycloakAdminClient>,
    mail_service: Data<MailService>,
    current_tenant: ReqData<Tenant>,
    current_user: ReqData<User>,
    path_params: Path<RoomAndStreamingTargetId>,
    query: Query<StreamingTargetOptionsQuery>,
    update_streaming_target: Json<ChangeRoomStreamingTargetRequest>,
) -> DefaultApiResult<ChangeRoomStreamingTargetResponse> {
    let settings = settings.load_full();
    let mail_service = mail_service.into_inner();
    let current_tenant = current_tenant.into_inner();
    let current_user = current_user.into_inner();
    let query = query.into_inner();
    let update_streaming_target = update_streaming_target.into_inner().0;

    if update_streaming_target.name.is_none() && update_streaming_target.kind.is_none() {
        return Err(ApiError::bad_request());
    }

    let send_email_notification = !query.suppress_email_notification;

    let mut conn = db.get_conn().await?;

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
                .map_err(|e| {
                    log::warn!(
                        "Invalid streaming endpoint url entry in db: {}",
                        Report::from_error(e)
                    );
                    ApiError::internal()
                })?,
            streaming_key: room_streaming_target_table.streaming_key,
            public_url: room_streaming_target_table
                .public_url
                .parse()
                .map_err(|e| {
                    log::warn!("Invalid public url entry in db: {}", Report::from_error(e));
                    ApiError::internal()
                })?,
        },
    };

    let room_streaming_target = RoomStreamingTarget {
        id: room_streaming_target_table.id,
        streaming_target: StreamingTarget {
            name: room_streaming_target_table.name,
            kind,
        },
    };

    if send_email_notification {
        notify_event_invitees_by_room_about_update(
            &kc_admin_client,
            settings,
            mail_service,
            current_tenant,
            current_user,
            &mut conn,
            room_id,
        )
        .await?;
    }

    Ok(ApiResponse::new(ChangeRoomStreamingTargetResponse(
        room_streaming_target,
    )))
}

/// Deletes a single streaming target.
///
/// The streaming target is deleted from the room.
#[utoipa::path(
    params(
        RoomAndStreamingTargetId,
        StreamingTargetOptionsQuery,
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "The streaming target has been deleted",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::FORBIDDEN,
            response = Forbidden,
        ),
        (
            status = StatusCode::NOT_FOUND,
            response = NotFound,
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[delete("/rooms/{room_id}/streaming_targets/{streaming_target_id}")]
#[allow(clippy::too_many_arguments)]
pub async fn delete_streaming_target(
    settings: SharedSettingsActix,
    db: Data<Db>,
    kc_admin_client: Data<KeycloakAdminClient>,
    mail_service: Data<MailService>,
    current_tenant: ReqData<Tenant>,
    current_user: ReqData<User>,
    path_params: Path<RoomAndStreamingTargetId>,
    query: Query<StreamingTargetOptionsQuery>,
) -> Result<NoContent, ApiError> {
    let settings = settings.load_full();
    let mail_service = mail_service.into_inner();
    let current_tenant = current_tenant.into_inner();
    let current_user = current_user.into_inner();
    let query = query.into_inner();

    let send_email_notification = !query.suppress_email_notification;

    let mut conn = db.get_conn().await?;

    let RoomAndStreamingTargetId {
        room_id,
        streaming_target_id,
    } = path_params.into_inner();

    RoomStreamingTargetRecord::delete_by_id(&mut conn, room_id, streaming_target_id).await?;

    if send_email_notification {
        notify_event_invitees_by_room_about_update(
            &kc_admin_client,
            settings,
            mail_service,
            current_tenant,
            current_user,
            &mut conn,
            room_id,
        )
        .await?;
    }

    Ok(NoContent)
}

fn build_resource(
    room_streaming_target: RoomStreamingTarget,
    with_streaming_key: bool,
) -> RoomStreamingTargetResource {
    let kind_resource = match room_streaming_target.streaming_target.kind {
        StreamingTargetKind::Custom {
            streaming_endpoint,
            streaming_key,
            public_url,
        } => StreamingTargetKindResource::Custom {
            streaming_endpoint,
            streaming_key: if with_streaming_key {
                Some(streaming_key)
            } else {
                None
            },
            public_url,
        },
    };

    RoomStreamingTargetResource {
        id: room_streaming_target.id,
        streaming_target: StreamingTargetResource {
            name: room_streaming_target.streaming_target.name,
            kind: kind_resource,
        },
    }
}
