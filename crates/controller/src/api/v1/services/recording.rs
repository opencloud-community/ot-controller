// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::api::signaling::ticket::start_or_continue_signaling_session;
use crate::api::v1::response::ApiError;
use crate::api::v1::response::NoContent;
use crate::settings::SharedSettingsActix;
use actix_web::dev::HttpServiceFactory;
use actix_web::post;
use actix_web::web::Payload;
use actix_web::web::Query;
use actix_web::web::{Data, Json};
use database::Db;
use db_storage::rooms::Room;
use futures::TryStreamExt;
use signaling_core::assets::save_asset;
use signaling_core::{ObjectStorage, Participant, RedisConnection};
use types::api::v1::services::UploadRenderQuery;
use types::api::v1::services::{ServiceStartResponse, StartBody};

// Note to devs:
// Please update `docs/admin/keycloak.md` service login documentation as well if
// you change something here
const REQUIRED_RECORDING_ROLE: &str = "opentalk-recorder";

#[post("/start")]
pub async fn start(
    settings: SharedSettingsActix,
    db: Data<Db>,
    redis_ctx: Data<RedisConnection>,
    body: Json<StartBody>,
) -> Result<Json<ServiceStartResponse>, ApiError> {
    let settings = settings.load_full();
    if settings.rabbit_mq.recording_task_queue.is_none() {
        return Err(ApiError::not_found());
    }

    let mut redis_conn = (**redis_ctx).clone();
    let body = body.into_inner();

    let room = Room::get(&mut db.get_conn().await?, body.room_id).await?;

    let (ticket, resumption) = start_or_continue_signaling_session(
        &mut redis_conn,
        Participant::Recorder,
        room.id,
        None,
        None,
    )
    .await?;

    Ok(Json(ServiceStartResponse { ticket, resumption }))
}

#[post("/upload_render")]
pub async fn upload_render(
    storage: Data<ObjectStorage>,
    db: Data<Db>,
    query: Query<UploadRenderQuery>,
    data: Payload,
) -> Result<NoContent, ApiError> {
    // Assert that the room exists
    Room::get(&mut db.get_conn().await?, query.room_id).await?;

    save_asset(
        &storage,
        db.into_inner(),
        query.room_id,
        Some("recording"),
        &query.filename,
        "recording-render",
        data.into_stream().map_err(anyhow::Error::from),
    )
    .await?;

    Ok(NoContent)
}

pub fn services() -> impl HttpServiceFactory {
    actix_web::web::scope("/recording")
        .wrap(super::RequiredRealmRole::new(REQUIRED_RECORDING_ROLE))
        .service(start)
        .service(upload_render)
}
