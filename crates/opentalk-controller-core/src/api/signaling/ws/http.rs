// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{BTreeMap, BTreeSet},
    marker::PhantomData,
    sync::Arc,
    time::Instant,
};

use actix_web::{get, http::header, web, web::Data, HttpMessage, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use kustos::Authz;
use opentalk_controller_service::signaling::{
    resumption::{ResumptionData, ResumptionTokenKeepAlive},
    storage::{SignalingStorage, SignalingStorageProvider as _},
    ticket::TicketData,
};
use opentalk_controller_settings::SettingsProvider;
use opentalk_controller_utils::CaptureApiError;
use opentalk_database::Db;
use opentalk_db_storage::{rooms::Room, users::User};
use opentalk_signaling_core::{
    ExchangeHandle, ObjectStorage, Participant, SignalingMetrics, SignalingModule, VolatileStorage,
};
use opentalk_types_api_v1::error::ApiError;
use opentalk_types_common::{
    auth::TicketToken,
    features::{FeatureId, ModuleFeatureId},
    modules::ModuleId,
    tariffs::TariffResource,
};
use snafu::Report;
use tokio::{
    sync::{broadcast, mpsc},
    task,
};
use tracing_actix_web::RequestId;

use super::{
    modules::{ModuleBuilder, ModuleBuilderImpl},
    runner::Runner,
};
use crate::api::{
    responses::{BadRequest, Forbidden, InternalServerError, Unauthorized},
    signaling::ws::actor::WebSocketActor,
};

#[derive(Default)]
pub struct SignalingModules(Vec<Box<dyn ModuleBuilder>>);

impl SignalingModules {
    pub fn add_module<M>(&mut self, params: M::Params)
    where
        M: SignalingModule + 'static,
    {
        self.0.push(Box::new(ModuleBuilderImpl {
            m: PhantomData::<fn() -> M>,
            params,
        }));
    }

    pub fn get_module_features(&self) -> BTreeMap<ModuleId, BTreeSet<FeatureId>> {
        self.0
            .iter()
            .map(|m| (m.module_id(), m.provided_features()))
            .collect()
    }
}

pub struct SignalingProtocols(&'static [&'static str]);

impl SignalingProtocols {
    pub fn data() -> Data<Self> {
        Data::new(Self(&["opentalk-signaling-json-v1.0"]))
    }
}

/// Room signaling websocket
///
/// The room signaling websocket.
/// Documentation: <https://docs.opentalk.eu/developer/controller/signaling/>.
#[utoipa::path(
    params(
        crate::api::headers::SignalingProtocolHeaders,
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "WebSocket connection succcessfully established",
        ),
        (
            status = StatusCode::BAD_REQUEST,
            response = BadRequest,
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
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[allow(clippy::too_many_arguments)]
#[get("/signaling")]
pub(crate) async fn ws_service(
    shutdown: Data<broadcast::Sender<()>>,
    db: Data<Db>,
    storage: Data<ObjectStorage>,
    authz: Data<Authz>,
    volatile: Data<VolatileStorage>,
    exchange_handle: Data<ExchangeHandle>,
    metrics: Data<SignalingMetrics>,
    protocols: Data<SignalingProtocols>,
    modules: Data<SignalingModules>,
    request: HttpRequest,
    stream: web::Payload,
    settings_provider: Data<SettingsProvider>,
) -> actix_web::Result<HttpResponse> {
    ws_service_inner(
        &shutdown,
        db.into_inner(),
        storage.into_inner(),
        authz.into_inner(),
        (**volatile).clone(),
        (**exchange_handle).clone(),
        metrics.into_inner(),
        &protocols,
        &modules,
        request,
        stream,
        (**settings_provider).clone(),
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn ws_service_inner(
    shutdown: &broadcast::Sender<()>,
    db: Arc<Db>,
    storage: Arc<ObjectStorage>,
    authz: Arc<Authz>,
    mut volatile: VolatileStorage,
    exchange_handle: ExchangeHandle,
    metrics: Arc<SignalingMetrics>,
    protocols: &SignalingProtocols,
    modules: &SignalingModules,
    request: HttpRequest,
    stream: web::Payload,
    settings_provider: SettingsProvider,
) -> actix_web::Result<HttpResponse> {
    let request_id = if let Some(request_id) = request.extensions().get::<RequestId>() {
        *request_id
    } else {
        log::error!("missing request id in signaling request");
        return Ok(HttpResponse::InternalServerError().finish());
    };

    // Read ticket and protocol from protocol header
    let (ticket, protocol) = read_request_header(&request, protocols.0)?;

    // Read ticket data from storage
    let ticket_data = take_ticket_data_from_storage(volatile.signaling_storage(), &ticket).await?;

    // Get user & room from database using the ticket data
    let (participant, room) = get_user_and_room_from_ticket_data(&db, &ticket_data).await?;

    // Create resumption data to be refreshed by the runner in volatile storage
    let resumption_data = ResumptionData {
        participant_id: ticket_data.participant_id,
        participant: match &participant {
            Participant::User(user) => Participant::User(user.id),
            Participant::Guest => Participant::Guest,
            Participant::Sip => Participant::Sip,
            Participant::Recorder => Participant::Recorder,
        },
        room: ticket_data.room,
        breakout_room: ticket_data.breakout_room,
    };

    let room_tariff = get_tariff_for_room(
        &db,
        &room,
        settings_provider
            .get_raw()
            .defaults
            .disabled_features
            .clone(),
        modules.get_module_features(),
    )
    .await?;

    // Create keep-alive util for resumption data
    let resumption_keep_alive =
        ResumptionTokenKeepAlive::new(ticket_data.resumption, resumption_data);

    // Finish websocket handshake
    let (sender, recv) = mpsc::unbounded_channel();
    let (addr, response) =
        ws::WsResponseBuilder::new(WebSocketActor::new(sender), &request, stream)
            .protocols(protocols.0)
            .start_with_addr()?;

    let mut builder = match Runner::builder(
        request_id.into(),
        ticket_data.participant_id,
        ticket_data.resuming,
        room,
        room_tariff,
        ticket_data.breakout_room,
        participant,
        protocol,
        metrics.clone(),
        db,
        storage,
        authz,
        volatile,
        exchange_handle,
        resumption_keep_alive,
    )
    .await
    {
        Ok(builder) => builder,
        Err(e) => {
            log::error!("Failed to initialize builder, {}", Report::from_error(e));

            return Ok(HttpResponse::InternalServerError().finish());
        }
    };

    let startup_start_time = Instant::now();

    // add all modules
    for module in modules.0.iter() {
        if let Err(e) = module.build(&mut builder).await {
            log::error!("Failed to initialize module, {}", Report::from_error(e));

            metrics.record_startup_time(startup_start_time.elapsed().as_secs_f64(), false);

            builder.abort().await;
            return Ok(HttpResponse::InternalServerError().finish());
        }
    }

    // Build and initialize the runner
    let runner = match builder
        .build(addr, recv, shutdown.subscribe(), settings_provider)
        .await
    {
        Ok(runner) => runner,
        Err(e) => {
            log::error!("Failed to initialize runner, {}", Report::from_error(e));

            metrics.record_startup_time(startup_start_time.elapsed().as_secs_f64(), false);

            return Ok(HttpResponse::InternalServerError().finish());
        }
    };

    // Spawn the runner task
    task::spawn_local(runner.run());

    metrics.record_startup_time(startup_start_time.elapsed().as_secs_f64(), true);

    Ok(response)
}

fn read_request_header(
    request: &HttpRequest,
    allowed_protocols: &'static [&'static str],
) -> Result<(TicketToken, &'static str), ApiError> {
    let Some(protocol_header) = request
        .headers()
        .get(header::SEC_WEBSOCKET_PROTOCOL)
        .and_then(|v| v.to_str().ok())
    else {
        log::debug!(
            "Rejecting websocket request, missing protocol header. Headers: {:?}",
            request.headers()
        );
        return Err(ApiError::bad_request()
            .with_code("missing_protocol")
            .with_message("Missing sec-websocket-protocol header"));
    };

    let protocol_parts = protocol_header
        .split(',')
        .map(str::trim)
        .collect::<Vec<&str>>();

    let Some(protocol) = protocol_parts.iter().find_map(|part| {
        allowed_protocols
            .iter()
            .find(|&allowed_protocol| allowed_protocol == part)
    }) else {
        log::debug!("Rejecting websocket request, invalid protocol header: {protocol_header:?}");
        return Err(ApiError::bad_request()
            .with_code("invalid_protocol")
            .with_message("Missing valid protocol"));
    };

    let Some(ticket) = protocol_parts
        .iter()
        .find_map(|part| part.strip_prefix("ticket#"))
    else {
        log::debug!("Rejecting websocket request, missing ticket in header '{protocol_header}'");
        return Err(ApiError::unauthorized()
            .with_code("missing_ticket")
            .with_message(
                "Missing ticket. Please request a new ticket from /v1/rooms/<room_id>/start",
            ));
    };

    if ticket.len() != 64 {
        log::warn!(
            "got ticket with invalid ticket length expected 64, got '{ticket}' ({ticket_len})",
            ticket_len = ticket.len()
        );
        return Err(ApiError::unauthorized()
            .with_code("invalid_ticket")
            .with_message(
                "Invalid ticket. Please request a new ticket from /v1/rooms/<room_id>/start",
            ));
    }

    Ok((TicketToken::from(ticket.to_string()), protocol))
}

async fn take_ticket_data_from_storage(
    storage: &mut dyn SignalingStorage,
    ticket: &TicketToken,
) -> Result<TicketData, ApiError> {
    let ticket_data = storage.take_ticket(ticket).await.map_err(|e| {
        log::warn!(
            "Unable to get ticket data in storage: {}",
            Report::from_error(e)
        );
        ApiError::internal()
    })?;

    let ticket_data = ticket_data.ok_or_else(|| {
        ApiError::unauthorized()
            .with_code("invalid_ticket")
            .with_message(
            "Invalid or expired ticket. Please request a new ticket from /v1/rooms/<room_id>/start",
        )
    })?;

    Ok(ticket_data)
}

async fn get_user_and_room_from_ticket_data(
    db: &Db,
    ticket_data: &TicketData,
) -> Result<(Participant<User>, Room), CaptureApiError> {
    let participant = ticket_data.participant;
    let room_id = ticket_data.room;

    let mut conn = db.get_conn().await?;

    let participant = match participant {
        Participant::User(user_id) => {
            let user = User::get(&mut conn, user_id).await?;

            Participant::User(user)
        }
        Participant::Guest => Participant::Guest,
        Participant::Sip => Participant::Sip,
        Participant::Recorder => Participant::Recorder,
    };

    let room = Room::get(&mut conn, room_id).await?;

    Ok((participant, room))
}

async fn get_tariff_for_room(
    db: &Db,
    room: &Room,
    disabled_features: BTreeSet<ModuleFeatureId>,
    module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
) -> Result<TariffResource, CaptureApiError> {
    let mut conn = db.get_conn().await?;

    let tariff = room.get_tariff(&mut conn).await?;

    Ok(tariff.to_tariff_resource(disabled_features, module_features))
}
