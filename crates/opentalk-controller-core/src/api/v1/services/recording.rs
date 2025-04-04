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
use opentalk_controller_service_facade::OpenTalkControllerService;
use opentalk_database::Db;
use opentalk_signaling_core::{
    assets::{save_asset, NewAssetFileName},
    ChunkFormat, ObjectStorage, ObjectStorageError,
};
use opentalk_types_api_v1::{
    error::{ApiError, ErrorBody},
    services::{
        recording::{GetRecordingUploadQuery, PostRecordingStartRequestBody},
        PostServiceStartResponseBody,
    },
};
use tokio::{sync::mpsc, task};

use crate::api::{
    headers::{ConnectionUpgrade, WebsocketUpgrade},
    responses::{InternalServerError, Unauthorized},
    upload::{UploadWebSocketActor, MAXIMUM_WEBSOCKET_BUFFER_SIZE},
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
            body = ErrorBody,
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
    service: Data<OpenTalkControllerService>,
    body: Json<PostRecordingStartRequestBody>,
) -> Result<Json<PostServiceStartResponseBody>, ApiError> {
    let response = service.start_recording(body.into_inner()).await?;

    Ok(Json(response))
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
        GetRecordingUploadQuery,
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
pub(crate) async fn get_recording_upload(
    db: Data<Db>,
    storage: Data<ObjectStorage>,
    request: HttpRequest,
    Query(GetRecordingUploadQuery {
        room_id,
        file_extension,
        timestamp,
    }): Query<GetRecordingUploadQuery>,
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
                Some(opentalk_types_signaling_recording::MODULE_ID),
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
        .service(get_recording_upload)
}
