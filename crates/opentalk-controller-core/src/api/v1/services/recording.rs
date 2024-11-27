// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_http::ws::Codec;
use actix_web::{
    dev::HttpServiceFactory,
    get, post, web,
    web::{Data, Json, Query},
    HttpRequest, HttpResponse,
};
use actix_web_actors::ws;
use bytes::Bytes;
use opentalk_database::Db;
use opentalk_db_storage::rooms::Room;
use opentalk_signaling_core::{
    assets::{save_asset, verify_storage_usage, NewAssetFileName},
    ChunkFormat, ObjectStorage, ObjectStorageError, Participant, VolatileStorage,
};
use opentalk_types::api::{error::ApiError, v1::services::UploadRenderQuery};
use opentalk_types_api_v1::services::{
    recording::PostRecordingStartRequestBody, PostServiceStartResponseBody,
};
use tokio::{sync::mpsc, task};

use crate::{
    api::{
        headers::{ConnectionUpgrade, WebsocketUpgrade},
        responses::{InternalServerError, Unauthorized},
        signaling::ticket::start_or_continue_signaling_session,
        upload::{UploadWebSocketActor, MAXIMUM_WEBSOCKET_BUFFER_SIZE},
    },
    settings::SharedSettingsActix,
};

// Note to devs:
// Please update `docs/admin/keycloak.md` service login documentation as well if
// you change something here
const REQUIRED_RECORDING_ROLE: &str = "opentalk-recorder";

/// Starts a signaling session for recording
///
/// This endpoint is provided for participation of recording and streaming clients
/// which will join incognito and receive all the information and media streams required
/// for creating a recording or livestream of the meeting.
#[utoipa::path(
    context_path = "/services/recording",
    request_body = PostRecordingStartRequestBody,
    operation_id = "start_recording",
    responses(
        (
            status = StatusCode::OK,
            description = "The recording participant has successfully \
                authenticated for the room. Information needed for connecting to the signaling \
                is contained in the response",
            body = PostServiceStartResponseBody,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::NOT_FOUND,
            description = "Recording has not been configured",
            body = StandardErrorBody,
            example = json!(ApiError::not_found().body),
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
#[post("/start")]
pub async fn post_recording_start(
    settings: SharedSettingsActix,
    db: Data<Db>,
    volatile: Data<VolatileStorage>,
    body: Json<PostRecordingStartRequestBody>,
) -> Result<Json<PostServiceStartResponseBody>, ApiError> {
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

    Ok(Json(PostServiceStartResponseBody { ticket, resumption }))
}

/// This is a dummy type to define the structure of the headers required for
/// upgrading a request to a recording upload websocket connection.
#[derive(utoipa::IntoParams)]
#[into_params(
    parameter_in = Header,
)]
#[allow(dead_code)]
pub(crate) struct RecordingUploadWebSocketHeaders {
    #[param(inline, required = true)]
    pub connection: ConnectionUpgrade,

    #[param(inline, required = true)]
    pub upgrade: WebsocketUpgrade,
}

/// Streaming upload of a rendered recording
///
/// This is a WebSocket endpoint, all the data that is sent in binary messages
/// is stored in the destination file.
#[utoipa::path(
    context_path = "/services/recording",
    params(
        UploadRenderQuery,
        RecordingUploadWebSocketHeaders,
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "WebSocket connection succcessfully established",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
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
                Some(opentalk_types_signaling_recording::NAMESPACE),
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
        .service(post_recording_start)
        .service(ws_upload)
}
