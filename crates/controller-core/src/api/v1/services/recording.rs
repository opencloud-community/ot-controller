// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_http::ws::Codec;
use actix_web::{
    dev::HttpServiceFactory,
    get, post, web,
    web::{Data, Json, Payload, Query},
    HttpRequest, HttpResponse,
};
use actix_web_actors::ws;
use bytes::Bytes;
use futures::TryStreamExt;
use opentalk_database::Db;
use opentalk_db_storage::rooms::Room;
use opentalk_signaling_core::{
    assets::{save_asset, verify_storage_usage, NewAssetFileName},
    ChunkFormat, ObjectStorage, ObjectStorageError, Participant, VolatileStorage,
};
use opentalk_types::api::{
    error::ApiError,
    v1::services::{ServiceStartResponse, StartBody, UploadRenderQuery},
};
use tokio::{sync::mpsc, task};

use crate::{
    api::{
        signaling::ticket::start_or_continue_signaling_session,
        upload::{UploadWebSocketActor, MAXIMUM_WEBSOCKET_BUFFER_SIZE},
        v1::response::NoContent,
    },
    settings::SharedSettingsActix,
};

// Note to devs:
// Please update `docs/admin/keycloak.md` service login documentation as well if
// you change something here
const REQUIRED_RECORDING_ROLE: &str = "opentalk-recorder";

#[post("/start")]
pub async fn start(
    settings: SharedSettingsActix,
    db: Data<Db>,
    volatile: Data<VolatileStorage>,
    body: Json<StartBody>,
) -> Result<Json<ServiceStartResponse>, ApiError> {
    let mut conn = db.get_conn().await?;
    let settings = settings.load_full();
    if settings.rabbit_mq.recording_task_queue.is_none() {
        return Err(ApiError::not_found());
    }

    let mut volatile = (**volatile).clone();
    let body = body.into_inner();

    let room = Room::get(&mut conn, body.room_id).await?;

    verify_storage_usage(&mut conn, room.created_by).await?;

    let (ticket, resumption) = start_or_continue_signaling_session(
        &mut volatile,
        Participant::Recorder,
        room.id,
        body.breakout_room,
        None,
    )
    .await?;

    Ok(Json(ServiceStartResponse { ticket, resumption }))
}

#[post("/upload_render")]
pub async fn upload_render(
    storage: Data<ObjectStorage>,
    db: Data<Db>,
    Query(UploadRenderQuery {
        room_id,
        file_extension,
        timestamp,
    }): Query<UploadRenderQuery>,
    data: Payload,
) -> Result<NoContent, ApiError> {
    // Assert that the room exists
    Room::get(&mut db.get_conn().await?, room_id).await?;

    let kind = "recording"
        .parse()
        .expect("Must be parseable as AssetFileKind");
    let filename = NewAssetFileName::new(kind, timestamp, file_extension);

    save_asset(
        &storage,
        db.into_inner(),
        room_id,
        Some("recording"),
        filename,
        data.into_stream(),
        ChunkFormat::Data,
    )
    .await?;

    Ok(NoContent)
}

#[get("/upload")]
pub(crate) async fn ws_upload(
    db: Data<Db>,
    storage: Data<ObjectStorage>,
    request: HttpRequest,
    Query(UploadRenderQuery {
        room_id,
        file_extension,
        timestamp,
    }): Query<UploadRenderQuery>,
    stream: web::Payload,
) -> actix_web::Result<HttpResponse> {
    // Finish websocket handshake
    let (sender, receiver) = mpsc::unbounded_channel::<Result<Bytes, ObjectStorageError>>();
    let receiver_stream = tokio_stream::wrappers::UnboundedReceiverStream::new(receiver);
    let (_addr, response) =
        ws::WsResponseBuilder::new(UploadWebSocketActor::new(sender), &request, stream)
            .codec(Codec::new().max_size(100_000_000))
            .frame_size(MAXIMUM_WEBSOCKET_BUFFER_SIZE)
            .start_with_addr()?;

    // Spawn the runner task
    task::spawn_local({
        async move {
            let kind = "recording"
                .parse()
                .expect("Must be parseable as AssetFileKind");
            let filename = NewAssetFileName::new(kind, timestamp, file_extension);

            let result = save_asset(
                &storage,
                db.into_inner(),
                room_id,
                Some(opentalk_types::signaling::recording::NAMESPACE),
                filename,
                receiver_stream,
                ChunkFormat::SequenceNumberAndData,
            )
            .await;

            if let Err(e) = result {
                log::error!("Error saving asset, {}", e);
            }
        }
    });

    Ok(response)
}

pub fn services() -> impl HttpServiceFactory {
    actix_web::web::scope("/recording")
        .wrap(super::RequiredRealmRole::new(REQUIRED_RECORDING_ROLE))
        .service(start)
        .service(upload_render)
        .service(ws_upload)
}
