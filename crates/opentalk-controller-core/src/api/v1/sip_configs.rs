// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use actix_web::{
    delete, get, put,
    web::{Data, Json, Path},
    HttpResponse,
};
use opentalk_controller_service::require_feature;
use opentalk_controller_settings::{OidcAndUserSearchConfiguration, SettingsLoading};
use opentalk_controller_utils::CaptureApiError;
use opentalk_database::Db;
use opentalk_db_storage::{
    rooms::Room,
    sip_configs::{NewSipConfig, SipConfig, UpdateSipConfig},
};
use opentalk_types_api_v1::{
    error::ApiError,
    rooms::by_room_id::sip::{PutSipConfigRequestBody, SipConfigResource},
};
use opentalk_types_common::{features, rooms::RoomId};

use crate::{
    api::responses::{Forbidden, InternalServerError, NotFound, Unauthorized},
    settings::SharedSettingsActix,
};

/// Get the sip config for the specified room.
///
/// Returns the sip config if available for the room, otherwise `404 NOT_FOUND`
/// is returned.
#[utoipa::path(
    operation_id = "get_room_sip",
    params(
        ("room_id" = RoomId, description = "The id of the room"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "The SIP config is successfully returned",
            body = SipConfigResource,
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
#[get("/rooms/{room_id}/sip")]
pub async fn get(
    settings: SharedSettingsActix,
    db: Data<Db>,
    room_id: Path<RoomId>,
) -> Result<Json<SipConfigResource>, ApiError> {
    Ok(get_inner(settings.load(), &db, room_id.into_inner()).await?)
}

async fn get_inner(
    settings: arc_swap::Guard<Arc<SettingsLoading<OidcAndUserSearchConfiguration>>>,
    db: &Db,
    room_id: RoomId,
) -> Result<Json<SipConfigResource>, CaptureApiError> {
    let mut conn = db.get_conn().await?;

    let room = Room::get(&mut conn, room_id).await?;

    if room.e2e_encryption {
        return Err(ApiError::not_found()
            .with_code("service_unavailable")
            .with_message("Call-in not available for end-to-end encrypted room".to_string())
            .into());
    }

    require_feature(
        &mut conn,
        &settings,
        room.created_by,
        &features::CALL_IN_MODULE_FEATURE_ID,
    )
    .await?;

    let config = SipConfig::get_by_room(&mut conn, room_id).await?;

    Ok(Json(SipConfigResource {
        room: room_id,
        sip_id: config.sip_id,
        password: config.password,
        lobby: config.lobby,
    }))
}

/// Modify the sip configuration of a room. A new sip configuration is created
/// if none was set before.
///
/// Returns the new modified sip configuration.
#[utoipa::path(
    params(
        ("room_id" = RoomId, description = "The id of the room"),
    ),
    request_body = PutSipConfigRequestBody,
    responses(
        (
            status = StatusCode::OK,
            description = "The SIP configuration was updated",
            body = SipConfigResource,
        ),
        (
            status = StatusCode::CREATED,
            description = "A new SIP configuration was created",
            body = SipConfigResource,
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
#[put("/rooms/{room_id}/sip")]
pub async fn put(
    settings: SharedSettingsActix,
    db: Data<Db>,
    room_id: Path<RoomId>,
    modify_sip_config: Json<PutSipConfigRequestBody>,
) -> Result<HttpResponse, ApiError> {
    Ok(put_inner(
        settings.load(),
        &db,
        room_id.into_inner(),
        modify_sip_config.into_inner(),
    )
    .await?)
}

async fn put_inner(
    settings: arc_swap::Guard<Arc<SettingsLoading<OidcAndUserSearchConfiguration>>>,
    db: &Db,
    room_id: RoomId,
    modify_sip_config: PutSipConfigRequestBody,
) -> Result<HttpResponse, CaptureApiError> {
    let mut conn = db.get_conn().await?;

    let room = Room::get(&mut conn, room_id).await?;

    if room.e2e_encryption {
        return Err(ApiError::forbidden()
            .with_code("service_unavailable")
            .with_message("Call-in not available for end-to-end encrypted room".to_string())
            .into());
    }

    require_feature(
        &mut conn,
        &settings,
        room.created_by,
        &features::CALL_IN_MODULE_FEATURE_ID,
    )
    .await?;

    let changeset = UpdateSipConfig {
        password: modify_sip_config.password.clone(),
        enable_lobby: modify_sip_config.lobby,
    };

    // FIXME: use on_conflict().do_update() (UPSERT) for this PUT
    // Try to modify the sip config before creating a new one
    let (sip_config, newly_created) =
        if let Some(db_sip_config) = changeset.apply(&mut conn, room_id).await? {
            let sip_config = SipConfigResource {
                room: room_id,
                sip_id: db_sip_config.sip_id,
                password: db_sip_config.password,
                lobby: db_sip_config.lobby,
            };

            (sip_config, false)
        } else {
            // Create a new sip config
            let mut new_config =
                NewSipConfig::new(room_id, modify_sip_config.lobby.unwrap_or_default());

            if let Some(password) = modify_sip_config.password {
                new_config.password = password;
            }

            let config = new_config.insert(&mut conn).await?;

            let config_resource = SipConfigResource {
                room: room_id,
                sip_id: config.sip_id,
                password: config.password,
                lobby: config.lobby,
            };

            (config_resource, true)
        };

    let mut response = if newly_created {
        HttpResponse::Created()
    } else {
        HttpResponse::Ok()
    };

    Ok(response.json(sip_config))
}

/// Delete the SIP configuration of a room.
///
/// This removes the dial-in functionality from the room.
#[utoipa::path(
    operation_id = "delete_room_sip",
    params(
        ("room_id" = RoomId, description = "The id of the room"),
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "The SIP configuration was successfully deleted",
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
#[delete("/rooms/{room_id}/sip")]
pub async fn delete(
    settings: SharedSettingsActix,
    db: Data<Db>,
    room_id: Path<RoomId>,
) -> Result<HttpResponse, ApiError> {
    Ok(delete_inner(settings.load(), &db, room_id.into_inner()).await?)
}
async fn delete_inner(
    settings: arc_swap::Guard<Arc<SettingsLoading<OidcAndUserSearchConfiguration>>>,
    db: &Db,
    room_id: RoomId,
) -> Result<HttpResponse, CaptureApiError> {
    let mut conn = db.get_conn().await?;

    let room = Room::get(&mut conn, room_id).await?;

    require_feature(
        &mut conn,
        &settings,
        room.created_by,
        &features::CALL_IN_MODULE_FEATURE_ID,
    )
    .await?;

    SipConfig::delete_by_room(&mut conn, room_id).await?;

    Ok(HttpResponse::NoContent().finish())
}
