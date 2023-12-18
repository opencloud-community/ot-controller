// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::settings::SharedSettingsActix;

use super::response::error::ApiError;
use super::util::require_feature;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, put, HttpResponse};
use database::Db;
use db_storage::rooms::Room;
use db_storage::sip_configs::{NewSipConfig, SipConfig, UpdateSipConfig};
use types::api::v1::rooms::sip_config_resource::{PutSipConfig, SipConfigResource};
use types::common::features;
use types::core::RoomId;
use validator::Validate;

/// API Endpoint *GET /rooms/{room_id}/sip*
///
/// Get the sip config for the specified room.
#[get("/rooms/{room_id}/sip")]
pub async fn get(
    settings: SharedSettingsActix,
    db: Data<Db>,
    room_id: Path<RoomId>,
) -> Result<Json<SipConfigResource>, ApiError> {
    let settings = settings.load();
    let room_id = room_id.into_inner();

    let mut conn = db.get_conn().await?;

    let room = Room::get(&mut conn, room_id).await?;

    require_feature(&mut conn, &settings, room.created_by, features::CALL_IN).await?;

    let config = SipConfig::get_by_room(&mut conn, room_id).await?;

    Ok(Json(SipConfigResource {
        room: room_id,
        sip_id: config.sip_id,
        password: config.password,
        lobby: config.lobby,
    }))
}

/// API Endpoint *PUT /rooms/{room_id}/sip*
///
/// Modifies a sip config with the provided [`PutSipConfig`]. A new sip config is created
/// when no config was set.
///
/// Returns the new modified sip config.
#[put("/rooms/{room_id}/sip")]
pub async fn put(
    settings: SharedSettingsActix,
    db: Data<Db>,
    room_id: Path<RoomId>,
    modify_sip_config: Json<PutSipConfig>,
) -> Result<HttpResponse, ApiError> {
    let settings = settings.load();
    let room_id = room_id.into_inner();
    let modify_sip_config = modify_sip_config.into_inner();

    modify_sip_config.validate()?;

    let mut conn = db.get_conn().await?;

    let room = Room::get(&mut conn, room_id).await?;

    require_feature(&mut conn, &settings, room.created_by, features::CALL_IN).await?;

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

/// API Endpoint *DELETE /rooms/{room_id}/sip*
///
/// Deletes the sip config of the provided room.
#[delete("/rooms/{room_id}/sip")]
pub async fn delete(
    settings: SharedSettingsActix,
    db: Data<Db>,
    room_id: Path<RoomId>,
) -> Result<HttpResponse, ApiError> {
    let settings = settings.load();
    let room_id = room_id.into_inner();

    let mut conn = db.get_conn().await?;

    let room = Room::get(&mut conn, room_id).await?;

    require_feature(&mut conn, &settings, room.created_by, features::CALL_IN).await?;

    SipConfig::delete_by_room(&mut conn, room_id).await?;

    Ok(HttpResponse::NoContent().finish())
}
