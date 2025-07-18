// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Extensible core library of the *OpentTalk Controller*
//!
//! # Example
//!
//! ```no_run
//! use opentalk_controller_core::Controller;
//! use opentalk_controller_service::Whatever;
//!
//! # use opentalk_signaling_core::{ModulesRegistrar, RegisterModules};
//! # struct Modules;
//! # #[async_trait::async_trait(?Send)]
//! # impl RegisterModules for Modules {
//! #     async fn register<E>(registrar: &mut impl ModulesRegistrar<Error=E>) -> Result<(), E> {
//! #         unimplemented!();
//! #     }
//! # }
//!
//! #[actix_web::main]
//! async fn main() {
//!     opentalk_controller_core::try_or_exit(run()).await;
//! }
//!
//! async fn run() -> Result<(), Whatever> {
//!    if let Some(controller) = Controller::create::<Modules>("OpenTalk Controller").await? {
//!         controller.run().await?;
//!     }
//!
//!     Ok(())
//! }
//! ```

use std::{
    fs::File,
    io::BufReader,
    marker::PhantomData,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, ToSocketAddrs as _},
    sync::Arc,
    time::Duration,
};

use actix_cors::Cors;
use actix_web::{App, HttpServer, Scope, web, web::Data};
use api::signaling::SignalingModules;
use async_trait::async_trait;
use kustos::Authz;
use lapin_pool::RabbitMqPool;
use opentalk_controller_service::{
    ControllerBackend, Whatever,
    oidc::OidcContext,
    services::MailService,
    signaling::ws_modules::{breakout::BreakoutRooms, echo::Echo, moderation::ModerationModule},
};
use opentalk_controller_service_facade::OpenTalkControllerService;
use opentalk_controller_settings::{
    HttpTls, Monitoring, Settings, SettingsProvider, UserSearchBackend, UserSearchBackendKeycloak,
};
use opentalk_database::Db;
use opentalk_jobs::job_runner::JobRunner;
use opentalk_keycloak_admin::{AuthorizedClient, KeycloakAdminClient};
use opentalk_roomserver_client::Client as RoomServerClient;
use opentalk_signaling_core::{
    ExchangeHandle, ExchangeTask, ModulesRegistrar, ObjectStorage, RedisConnection,
    RegisterModules, SignalingModule, SignalingModuleInitData, VolatileStaticMemoryStorage,
    VolatileStorage,
};
use opentalk_types_api_v1::{auth::OidcProvider, error::ApiError};
use rustls_pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use service_probe::{ServiceState, set_service_state, start_probe};
use snafu::{ErrorCompat, Report, ResultExt, Snafu};
use swagger::WithSwagger as _;
use tokio::{
    signal::{
        ctrl_c,
        unix::{SignalKind, signal},
    },
    sync::broadcast,
    task::JoinError,
    time::sleep,
};
use tracing_actix_web::TracingLogger;

use crate::{
    acl::check_or_create_kustos_default_permissions,
    api::{
        signaling::SignalingProtocols,
        v1::{middleware::metrics::RequestMetrics, response::error::json_error_handler},
    },
    trace::ReducedSpanBuilder,
};

mod acl;
mod caches;
mod cli;
mod metrics;
mod swagger;
mod trace;

pub mod api;
pub mod settings;

#[derive(Debug, Snafu)]
/// Blocking thread has panicked
pub struct BlockingError {
    source: JoinError,
}

impl From<BlockingError> for ApiError {
    fn from(e: BlockingError) -> Self {
        log::error!(
            "REST API threw internal error from blocking error: {}",
            Report::from_error(e)
        );
        Self::internal()
    }
}

type Result<T, E = Whatever> = std::result::Result<T, E>;

/// Custom version of `actix_web::web::block` which retains the current tracing span
pub async fn block<F, R>(f: F) -> Result<R, BlockingError>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let span = tracing::Span::current();

    let fut = actix_rt::task::spawn_blocking(move || span.in_scope(f));

    fut.await.context(BlockingSnafu)
}

/// Wrapper of the main function. Correctly outputs the error to the logging utility or stderr.
pub async fn try_or_exit<T, F>(f: F) -> T
where
    F: std::future::Future<Output = Result<T>>,
{
    match f.await {
        Ok(ok) => ok,
        Err(err) => {
            let show_backtrace = std::env::var("RUST_BACKTRACE").is_ok_and(|v| v != "0");

            let backtrace = if show_backtrace {
                err.backtrace()
                    .map(|e| format!("\nBacktrace:\n{e}"))
                    .unwrap_or_else(|| "No backtrace available".to_string())
            } else {
                "NOTE: run with `RUST_BACKTRACE=1` environment variable to display a backtrace"
                    .to_string()
            };

            let report = Report::from_error(err);

            let message = format!("Error: {report}{backtrace}");

            if log::log_enabled!(log::Level::Error) {
                log::error!("{message}");
            } else {
                eprintln!("{message}");
            }

            std::process::exit(-1);
        }
    }
}

struct ControllerModules<M: RegisterModules>(PhantomData<M>);

#[async_trait(?Send)]
impl<M: RegisterModules> RegisterModules for ControllerModules<M> {
    async fn register<E>(
        registrar: &mut impl ModulesRegistrar<Error = E>,
    ) -> std::result::Result<(), E> {
        registrar.register::<Echo>().await?;
        registrar.register::<BreakoutRooms>().await?;
        registrar.register::<ModerationModule>().await?;
        M::register(registrar).await
    }
}

/// Controller struct representation containing all fields required to extend and drive the controller
pub struct Controller {
    pub service: OpenTalkControllerService,

    /// Settings loaded on [Controller::create]
    pub startup_settings: Arc<Settings>,

    /// Cloneable shared settings, can be used to reload settings from, when receiving the `reload` signal.
    pub settings_provider: SettingsProvider,

    /// CLI arguments
    args: cli::Args,

    db: Arc<Db>,

    storage: Arc<ObjectStorage>,

    oidc: Arc<OidcContext>,

    user_search_client: Arc<Option<KeycloakAdminClient>>,

    authz: Authz,

    /// RabbitMQ connection pool, can be used to create connections and channels
    pub rabbitmq_pool: Arc<Option<Arc<RabbitMqPool>>>,

    /// Handle to the internal message exchange
    pub exchange_handle: ExchangeHandle,

    /// Cloneable volatile storage
    pub volatile: VolatileStorage,

    /// Reload signal which can be triggered by a user.
    /// When received a module should try to re-read it's config and act accordingly.
    ///
    /// `controller.reload.subscribe()` to receive a receiver to the reload-signal.
    pub reload: broadcast::Sender<()>,

    /// Shutdown signal which is triggered when the controller is exiting, either because a fatal error occurred
    /// or a user requested the shutdown.
    ///
    /// `controller.shutdown.subscribe()` to receive a receiver to the reload-signal.
    /// The controller will wait up 10 seconds before forcefully shutting down.
    /// It is tracking the shutdown progress by counting the shutdown-receiver count.
    pub shutdown: broadcast::Sender<()>,

    /// List of signaling modules registered to the controller.
    ///
    /// Can and should be used to extend the controllers signaling endpoint's capabilities.
    pub signaling_modules: SignalingModules,

    /// All metrics of the Application
    pub metrics: metrics::CombinedMetrics,
}

impl Controller {
    /// Tries to create a controller from CLI arguments and then the settings.
    ///
    /// This can return Ok(None) which would indicate that the controller executed a CLI
    /// subprogram (e.g. `--reload`) and must now exit.
    ///
    /// Otherwise it will return itself which can be modified and then run using [`Controller::run`]
    pub async fn create<M: RegisterModules>(program_name: &str) -> Result<Option<Self>> {
        let args = cli::parse_args::<ControllerModules<M>>()
            .await
            .whatever_context("Failed to parse cli arguments")?;

        // Some args run commands by them self and thus should exit here
        if !args.controller_should_start() {
            return Ok(None);
        }

        let settings_provider =
            SettingsProvider::load_from_path_or_standard_paths(args.config.as_deref())
                .whatever_context("Failed to load settings")?;
        let settings = settings_provider.get();

        trace::init(&settings.logging).whatever_context("Failed to initialize tracing")?;

        log::info!("Starting {}", program_name);

        let controller = Self::init::<ControllerModules<M>>(settings_provider, args)
            .await
            .whatever_context("Failed to init controller")?;

        Ok(Some(controller))
    }

    #[tracing::instrument(err, skip(settings_provider, args))]
    async fn init<M: RegisterModules>(
        settings_provider: SettingsProvider,
        args: cli::Args,
    ) -> Result<Self> {
        let settings = settings_provider.get();
        let metrics = metrics::CombinedMetrics::try_init()
            .whatever_context("Failed to initialize metrics")?;

        opentalk_db_storage::migrations::migrate_from_url(&settings.database.url)
            .await
            .whatever_context("Failed to migrate database")?;

        let rabbitmq_pool = Arc::new(settings.rabbit_mq.as_ref().map(|config| {
            RabbitMqPool::from_config(
                &config.url,
                config.min_connections,
                config.max_channels_per_connection,
            )
        }));

        // Only use rabbitmq in the exchange when a both rabbitmq and redis are configured.
        // This assumes that the existence of redis means multiple controllers are used
        // and share their signaling state via redis. Only in this case rabbitmq is required
        // in the exchange.
        let exchange_handle = match (settings.redis.is_some(), rabbitmq_pool.as_ref()) {
            (true, Some(rabbitmq_pool)) => ExchangeTask::spawn_with_rabbitmq(rabbitmq_pool.clone())
                .await
                .whatever_context("Failed to spawn exchange task")?,
            _ => ExchangeTask::spawn()
                .await
                .whatever_context("Failed to spawn exchange task")?,
        };

        // Connect to postgres
        let mut db =
            Db::connect(&settings.database).whatever_context("Failed to connect to database")?;
        db.set_metrics(metrics.database.clone());
        let db = Arc::new(db);

        // Connect to MinIO
        let storage = Arc::new(
            ObjectStorage::new(&settings.minio)
                .await
                .whatever_context("Failed to initialize object storage")?,
        );

        let oidc_frontend = &settings.oidc.frontend;
        let oidc_controller = &settings.oidc.controller;

        // Discover OIDC Provider
        let oidc = Arc::new(
            OidcContext::new(
                oidc_frontend.authority.clone(),
                oidc_controller.authority.clone(),
                oidc_controller.client_id.clone(),
                oidc_controller.client_secret.clone(),
            )
            .await
            .whatever_context("Failed to initialize OIDC Context")?,
        );

        let user_search_client =
            if let Some(UserSearchBackend::Keycloak(UserSearchBackendKeycloak {
                api_base_url,
                client_id,
                client_secret,
                external_id_user_attribute_name: _,
            })) = &settings.user_search_backend
            {
                let authorized_client = AuthorizedClient::new(
                    oidc_controller.authority.clone(),
                    client_id.clone().into(),
                    client_secret.secret().clone(),
                )
                .whatever_context("Failed to initialize authorized client")?;

                Arc::new(Some(
                    KeycloakAdminClient::new(api_base_url.clone(), authorized_client)
                        .whatever_context("Failed to initialize keycloak")?,
                ))
            } else {
                Arc::new(None)
            };

        // Build redis client. Does not check if redis is reachable.
        let redis = settings
            .redis
            .as_ref()
            .map(|r| redis::Client::open(r.url.clone()))
            .transpose()
            .whatever_context("Invalid redis url")?;
        let redis_conn = match redis {
            Some(c) => Some(
                redis::aio::ConnectionManager::new(c)
                    .await
                    .whatever_context("Failed to create redis connection manager")?,
            ),
            None => None,
        };
        let redis_conn =
            redis_conn.map(|c| RedisConnection::new(c).with_metrics(metrics.redis.clone()));
        let volatile = match redis_conn {
            Some(redis) => VolatileStorage::Right(redis),
            None => VolatileStorage::Left(VolatileStaticMemoryStorage),
        };

        let (shutdown, _) = broadcast::channel::<()>(1);
        let (reload, _) = broadcast::channel::<()>(4);

        let authz = match (
            settings.authz.synchronize_controllers,
            rabbitmq_pool.as_ref(),
        ) {
            (true, Some(rabbitmq_pool)) => kustos::Authz::new_with_autoload_and_metrics(
                db.clone(),
                rabbitmq_pool.clone(),
                metrics.kustos.clone(),
            )
            .await
            .whatever_context("Failed to initialize kustos/authz")?,
            _ => kustos::Authz::new(db.clone())
                .await
                .whatever_context("Failed to initialize kustos/authz")?,
        };

        let mail_service = Arc::new(match rabbitmq_pool.as_ref() {
            Some(rabbitmq_pool) => Some(MailService::new(
                metrics.endpoint.clone(),
                rabbitmq_pool.clone(),
                rabbitmq_pool
                    .create_channel()
                    .await
                    .whatever_context("Failed to create rabbitmq channel")?,
            )),
            None => None,
        });

        let mut initializer = ModuleInitializer {
            init_data: SignalingModuleInitData {
                startup_settings: settings.clone(),
                settings_provider: settings_provider.clone(),
                rabbitmq_pool: rabbitmq_pool.clone(),
                volatile: volatile.clone(),
                shutdown: shutdown.clone(),
                reload: reload.clone(),
            },
            signaling_modules: SignalingModules::default(),
        };

        M::register(&mut initializer)
            .await
            .whatever_context("Failed to register modules")?;

        let roomserver_client = if let Some(roomserver_config) = &settings.roomserver {
            Some(
                RoomServerClient::new(
                    roomserver_config.url.clone(),
                    roomserver_config.api_token.clone(),
                )
                .whatever_context("Failed to create roomserver client")?,
            )
        } else {
            None
        };

        let backend = {
            let oidc_provider = OidcProvider {
                name: oidc_frontend.client_id.to_string(),
                url: oidc_frontend.authority.to_string(),
            };

            ControllerBackend::new(
                settings_provider.clone(),
                authz.clone(),
                db.clone(),
                oidc_provider,
                storage.clone(),
                volatile.clone(),
                exchange_handle.clone(),
                mail_service.clone(),
                user_search_client.clone(),
                initializer.signaling_modules.get_module_features(),
                roomserver_client,
            )
        };
        let service = OpenTalkControllerService::new(backend);

        let controller = Self {
            service,
            startup_settings: settings,
            settings_provider,
            args,
            db,
            storage,
            oidc,
            user_search_client,
            authz,
            rabbitmq_pool,
            exchange_handle,
            volatile,
            shutdown,
            reload,
            signaling_modules: initializer.signaling_modules,
            metrics,
        };

        Ok(controller)
    }

    /// Runs the controller until a fatal error occurred or a shutdown is requested (e.g. SIGTERM).
    pub async fn run(self) -> Result<()> {
        let signaling_modules = Arc::new(self.signaling_modules);

        if let Some(Monitoring { port, addr }) = self.startup_settings.monitoring {
            start_probe(addr, port, ServiceState::Up)
                .await
                .whatever_context("Failed to start monitoring")?;
        }

        // Start JobExecutor
        JobRunner::start(
            self.db.clone(),
            self.shutdown.subscribe(),
            self.startup_settings.clone(),
            self.exchange_handle.clone(),
        )
        .await
        .whatever_context("Failed to start Job Runner")?;

        // Start HTTP Server
        let http_server = {
            let settings_provider = self.settings_provider.clone();
            let volatile = self.volatile.clone();
            let exchange_handle = Data::new(self.exchange_handle);
            let signaling_modules = Arc::downgrade(&signaling_modules);
            let signaling_metrics = Data::from(self.metrics.signaling.clone());
            let db = Arc::downgrade(&self.db);
            let storage = Arc::downgrade(&self.storage);

            let oidc_ctx = Arc::downgrade(&self.oidc);
            let shutdown = self.shutdown.clone();

            let user_search_client = Data::from(self.user_search_client);

            log::info!("Making sure the default permissions are set");
            check_or_create_kustos_default_permissions(&self.authz)
                .await
                .whatever_context("Failed to create default permissions")?;

            let authz_middleware = self.authz.actix_web_middleware(true).await;

            let metrics = Data::new(self.metrics);

            let caches = Data::new(caches::Caches::create(self.volatile.right().clone()));
            let service = Data::new(self.service.clone());

            HttpServer::new(move || {
                let cors = setup_cors();

                // Unwraps cannot panic. Server gets stopped before dropping the Arc.
                let db = Data::from(db.upgrade().unwrap());
                let storage = Data::from(storage.upgrade().unwrap());

                let oidc_ctx = Data::from(oidc_ctx.upgrade().unwrap());
                let authz = Data::new(self.authz.clone());
                let volatile = Data::new(volatile.clone());

                let acl = authz_middleware.clone();

                let signaling_modules = Data::from(signaling_modules.upgrade().unwrap());
                let swagger_service_enabled = !settings_provider.get().endpoints.disable_openapi;

                App::new()
                    .wrap(RequestMetrics::new(metrics.endpoint.clone()))
                    .wrap(cors)
                    .wrap(TracingLogger::<ReducedSpanBuilder>::new())
                    .wrap(api::v1::middleware::headers::Headers {})
                    .app_data(service.clone())
                    .app_data(caches.clone())
                    .app_data(web::JsonConfig::default().error_handler(json_error_handler))
                    .app_data(Data::new(settings_provider.clone()))
                    .app_data(db.clone())
                    .app_data(storage)
                    .app_data(oidc_ctx.clone())
                    .app_data(user_search_client.clone())
                    .app_data(authz.clone())
                    .app_data(volatile)
                    .app_data(Data::new(shutdown.clone()))
                    .app_data(exchange_handle.clone())
                    .app_data(signaling_modules)
                    .app_data(SignalingProtocols::data())
                    .app_data(signaling_metrics.clone())
                    .app_data(metrics.clone())
                    .service(api::well_known::well_known_api)
                    .service(api::signaling::ws_service)
                    .service(metrics::metrics)
                    .with_swagger_service_if(swagger_service_enabled)
                    .service(v1_scope(
                        settings_provider.clone(),
                        authz,
                        db.clone(),
                        oidc_ctx.clone(),
                        acl,
                    ))
            })
        };

        let socket_address = determine_socket_address(
            self.startup_settings.http.addr.as_deref(),
            self.startup_settings.http.port,
        )
        .whatever_context("Unable to determine bind address")?;

        let http_server = if let Some(tls) = &self.startup_settings.http.tls {
            let config = setup_rustls(tls).whatever_context("Failed to setup TLS context")?;

            http_server.bind_rustls_0_23(&socket_address[..], config)
        } else {
            http_server.bind(&socket_address[..])
        };

        let http_server = http_server.with_whatever_context(|_| {
            format!("Failed to bind http server to {socket_address:?}")
        })?;

        set_service_state(ServiceState::Ready);
        log::info!("Startup finished");

        let http_server = http_server.disable_signals().run();
        let http_server_handle = http_server.handle();

        let mut reload_signal = signal(SignalKind::hangup())
            .whatever_context("Failed to register SIGHUP signal handler")?;

        actix_rt::spawn(http_server);

        // Wait for either SIGTERM or SIGHUP and handle them accordingly
        loop {
            tokio::select! {
                _ = ctrl_c() => {
                    log::info!("Got termination signal, exiting");
                    break;
                }
                _ = reload_signal.recv() => {
                    log::info!("Got reload signal, reloading");

                    if let Err(e) = self.settings_provider.reload_from_path_or_standard_paths(self.args.config.as_deref()) {
                        log::error!("Failed to reload settings, {}", Report::from_error(e));
                        continue
                    }

                    // discard result, might fail if no one is subscribed
                    let _ = self.reload.send(());
                }
            }
        }

        // ==== Begin shutdown sequence ====

        // Send shutdown signals to all tasks within our application
        let _ = self.shutdown.send(());

        // then stop HTTP server
        http_server_handle.stop(true).await;

        // Check in a 1 second interval for 10 seconds if all tasks have exited
        // by inspecting the receiver count of the broadcast-channel
        for _ in 0..10 {
            let receiver_count = self.shutdown.receiver_count();

            if receiver_count > 0 {
                log::debug!("Waiting for {} tasks to be stopped", receiver_count);
                sleep(Duration::from_secs(1)).await;
            }
        }

        // Drop signaling modules to drop any data contained in the module builders.
        drop(signaling_modules);

        if let Some(rabbitmq_pool) = self.rabbitmq_pool.as_ref() {
            // Close all rabbitmq connections
            // TODO what code and text to use here
            if let Err(e) = rabbitmq_pool.close(0, "shutting down").await {
                log::error!(
                    "Failed to close RabbitMQ connections, {}",
                    Report::from_error(e)
                );
            }
        }

        if self.shutdown.receiver_count() > 0 {
            log::error!("Not all tasks stopped. Exiting anyway");
        } else {
            log::info!("All tasks stopped, goodbye!");
        }

        Ok(())
    }
}

#[async_trait(?Send)]
impl ModulesRegistrar for Controller {
    type Error = Whatever;

    async fn register<M: SignalingModule>(&mut self) -> Result<()> {
        let init = SignalingModuleInitData {
            startup_settings: self.startup_settings.clone(),
            settings_provider: self.settings_provider.clone(),
            rabbitmq_pool: self.rabbitmq_pool.clone(),
            shutdown: self.shutdown.clone(),
            reload: self.reload.clone(),
            volatile: self.volatile.clone(),
        };

        let params = M::build_params(init)
            .await
            .with_whatever_context(|_| format!("Failed to initialize module '{}'", M::NAMESPACE))?;

        if let Some(params) = params {
            self.signaling_modules.add_module::<M>(params);
        } else {
            log::info!(
                "Skipping module '{}' due to missing configuration",
                M::NAMESPACE
            );
        }

        Ok(())
    }
}

#[derive(utoipa::OpenApi)]
#[openapi(
    info(
        title = "OpenTalk Controller Frontend API",
        description = "Specifies the endpoints and structure of the OpenTalk Controller Frontend API",
    ),
    tags(
        (
            name = "api::v1::auth",
            description = "Endpoints related to authentication"
        ),
        (
            name = "api::v1::invites",
            description = "Endpoints related to meeting invites"
        ),
        (
            name = "api::v1::rooms",
            description = "Endpoints related to meeting rooms"
        ),
        (
            name = "api::v1::events",
            description = "Endpoints related to events"
        ),
        (
            name = "api::v1::events::favorites",
            description = "Endpoints related to user's favorite events"
        ),
        (
            name = "api::v1::events::instances",
            description = "Endpoints related to event instances"
        ),
        (
            name = "api::v1::events::invites",
            description = "Endpoints related to event invites"
        ),
        (
            name = "api::v1::events::shared_folder",
            description = "Endpoints related to event shared folders"
        ),
        (
            name = "api::v1::assets",
            description = "Endpoints related to file assets"
        ),
        (
            name = "api::v1::sip_configs",
            description = "Endpoints related to SIP configuration"
        ),
        (
            name = "api::v1::services::call_in",
            description = "Endpoints related to the meeting call-in service"
        ),
        (
            name = "api::v1::services::recording",
            description = "Endpoints related to the meeting recording service"
        ),
        (
            name = "api::v1::streaming_targets",
            description = "Endpoints related to streaming targets"
        ),
        (
            name = "api::v1::turn",
            description = "Endpoints related TURN server usage"
        ),
        (
            name = "api::v1::users",
            description = "Endpoints related to user information and management"
        ),
        (
            name = "api::signaling",
            description = "Endpoints for signaling connections in a meeting"
        ),
    ),
    paths(
        api::signaling::ws_service,
        api::v1::assets::room_asset,
        api::v1::assets::room_assets,
        api::v1::assets::create,
        api::v1::assets::delete,
        api::v1::auth::get_login,
        api::v1::auth::post_login,
        api::v1::events::delete_event,
        api::v1::events::favorites::add_event_to_favorites,
        api::v1::events::favorites::remove_event_from_favorites,
        api::v1::events::get_event,
        api::v1::events::get_events,
        api::v1::events::instances::get_event_instance,
        api::v1::events::instances::get_event_instances,
        api::v1::events::instances::patch_event_instance,
        api::v1::events::invites::accept_event_invite,
        api::v1::events::invites::create_invite_to_event,
        api::v1::events::invites::decline_event_invite,
        api::v1::events::invites::delete_email_invite_to_event,
        api::v1::events::invites::delete_invite_to_event,
        api::v1::events::invites::get_event_invites_pending,
        api::v1::events::invites::get_invites_for_event,
        api::v1::events::invites::update_email_invite_to_event,
        api::v1::events::invites::update_invite_to_event,
        api::v1::events::new_event,
        api::v1::events::patch_event,
        api::v1::events::shared_folder::delete_shared_folder_for_event,
        api::v1::events::shared_folder::get_shared_folder_for_event,
        api::v1::events::shared_folder::put_shared_folder_for_event,
        api::v1::invites::add_invite,
        api::v1::invites::delete_invite,
        api::v1::invites::get_invite,
        api::v1::invites::get_invites,
        api::v1::invites::update_invite,
        api::v1::invites::verify_invite_code,
        api::v1::rooms::accessible,
        api::v1::rooms::delete,
        api::v1::rooms::get,
        api::v1::rooms::get_room_event,
        api::v1::rooms::get_room_tariff,
        api::v1::rooms::new,
        api::v1::rooms::patch,
        api::v1::rooms::start,
        api::v1::rooms::start_invited,
        api::v1::services::call_in::post_call_in_start,
        api::v1::services::recording::get_recording_upload,
        api::v1::services::recording::post_recording_start,
        api::v1::sip_configs::delete,
        api::v1::sip_configs::get,
        api::v1::sip_configs::put,
        api::v1::streaming_targets::delete_streaming_target,
        api::v1::streaming_targets::get_streaming_target,
        api::v1::streaming_targets::get_streaming_targets,
        api::v1::streaming_targets::patch_streaming_target,
        api::v1::streaming_targets::post_streaming_target,
        api::v1::turn::get,
        api::v1::users::find,
        api::v1::users::get_me,
        api::v1::users::get_me_assets,
        api::v1::users::get_me_tariff,
        api::v1::users::get_user,
        api::v1::users::patch_me,
    ),
    components(
        schemas(
            api::headers::CursorLink,
            api::headers::PageLink,
            opentalk_types_api_v1::error::ErrorBody,
            opentalk_types_api_v1::error::ValidationErrorEntry,
            opentalk_types_api_v1::Cursor::<opentalk_types_api_v1::events::GetEventInstancesCursorData>,
            opentalk_types_api_v1::Cursor::<opentalk_types_api_v1::events::GetEventsCursorData>,
            opentalk_types_api_v1::assets::AssetResource,
            opentalk_types_api_v1::assets::AssetSortingQuery,
            opentalk_types_api_v1::auth::GetLoginResponseBody,
            opentalk_types_api_v1::auth::OidcProvider,
            opentalk_types_api_v1::auth::PostLoginResponseBody,
            opentalk_types_api_v1::auth::login::AuthLoginPostRequestBody,
            opentalk_types_api_v1::events::CallInInfo,
            opentalk_types_api_v1::events::DeleteEmailInviteBody,
            opentalk_types_api_v1::events::EmailInvite,
            opentalk_types_api_v1::rooms::by_room_id::assets::RoomsByRoomIdAssetsGetResponseBody,
            opentalk_types_api_v1::events::EmailOnlyUser,
            opentalk_types_api_v1::events::EventAndInstanceId,
            opentalk_types_api_v1::events::EventExceptionResource,
            opentalk_types_api_v1::events::EventInstance,
            opentalk_types_api_v1::events::EventInvitee,
            opentalk_types_api_v1::events::EventInviteeProfile,
            opentalk_types_api_v1::events::EventOrException,
            opentalk_types_api_v1::events::EventResource,
            opentalk_types_api_v1::events::EventRoomInfo,
            opentalk_types_api_v1::events::EventStatus,
            opentalk_types_api_v1::events::EventType,
            opentalk_types_api_v1::events::GetEventInstanceResponseBody,
            opentalk_types_api_v1::events::GetEventInstancesResponseBody,
            opentalk_types_api_v1::events::InstanceId,
            opentalk_types_api_v1::events::PatchEmailInviteBody,
            opentalk_types_api_v1::events::PatchEventBody,
            opentalk_types_api_v1::events::PatchEventInstanceBody,
            opentalk_types_api_v1::events::PatchInviteBody,
            opentalk_types_api_v1::events::PostEventInviteBody,
            opentalk_types_api_v1::events::PostEventsBody,
            opentalk_types_api_v1::events::PublicInviteUserProfile,
            opentalk_types_api_v1::events::UserInvite,
            opentalk_types_api_v1::rooms::GetRoomsResponseBody,
            opentalk_types_api_v1::rooms::PostRoomsRequestBody,
            opentalk_types_api_v1::rooms::RoomResource,
            opentalk_types_api_v1::rooms::by_room_id::PatchRoomsRequestBody,
            opentalk_types_api_v1::rooms::by_room_id::PostRoomsStartInvitedRequestBody,
            opentalk_types_api_v1::rooms::by_room_id::PostRoomsStartRequestBody,
            opentalk_types_api_v1::rooms::by_room_id::RoomsStartResponseBody,
            opentalk_types_api_v1::rooms::by_room_id::invites::GetRoomsInvitesResponseBody,
            opentalk_types_api_v1::rooms::by_room_id::invites::InviteResource,
            opentalk_types_api_v1::rooms::by_room_id::invites::PostInviteRequestBody,
            opentalk_types_api_v1::rooms::by_room_id::invites::PostInviteVerifyRequestBody,
            opentalk_types_api_v1::rooms::by_room_id::invites::PostInviteVerifyResponseBody,
            opentalk_types_api_v1::rooms::by_room_id::invites::PutInviteRequestBody,
            opentalk_types_api_v1::rooms::by_room_id::sip::PutSipConfigRequestBody,
            opentalk_types_api_v1::rooms::by_room_id::sip::SipConfigResource,
            opentalk_types_api_v1::rooms::by_room_id::streaming_targets::GetRoomStreamingTargetResponseBody,
            opentalk_types_api_v1::rooms::by_room_id::streaming_targets::GetRoomStreamingTargetsResponseBody,
            opentalk_types_api_v1::rooms::by_room_id::streaming_targets::PatchRoomStreamingTargetRequestBody,
            opentalk_types_api_v1::rooms::by_room_id::streaming_targets::PatchRoomStreamingTargetResponseBody,
            opentalk_types_api_v1::rooms::by_room_id::streaming_targets::PostRoomStreamingTargetRequestBody,
            opentalk_types_api_v1::rooms::by_room_id::streaming_targets::PostRoomStreamingTargetResponseBody,
            opentalk_types_api_v1::rooms::streaming_targets::UpdateStreamingTargetKind,
            opentalk_types_api_v1::services::PostServiceStartResponseBody,
            opentalk_types_api_v1::services::call_in::PostCallInStartRequestBody,
            opentalk_types_api_v1::services::recording::PostRecordingStartRequestBody,
            opentalk_types_api_v1::users::GetEventInvitesPendingResponseBody,
            opentalk_types_api_v1::users::GetFindResponseBody,
            opentalk_types_api_v1::users::GetFindResponseEntry,
            opentalk_types_api_v1::users::GetUserAssetsResponseBody,
            opentalk_types_api_v1::users::PrivateUserProfile,
            opentalk_types_api_v1::users::PublicUserProfile,
            opentalk_types_api_v1::users::UnregisteredUser,
            opentalk_types_api_v1::users::UserAssetResource,
            opentalk_types_api_v1::users::me::PatchMeRequestBody,
            opentalk_types_common::assets::AssetId,
            opentalk_types_common::assets::AssetFileKind,
            opentalk_types_common::assets::AssetSorting,
            opentalk_types_common::assets::FileExtension,
            opentalk_types_common::auth::ResumptionToken,
            opentalk_types_common::auth::TicketToken,
            opentalk_types_common::call_in::CallInId,
            opentalk_types_common::call_in::CallInInfo,
            opentalk_types_common::call_in::CallInPassword,
            opentalk_types_common::call_in::NumericId,
            opentalk_types_common::email::EmailAddress,
            opentalk_types_common::events::EventDescription,
            opentalk_types_common::events::EventId,
            opentalk_types_common::events::EventInfo,
            opentalk_types_common::events::EventTitle,
            opentalk_types_common::events::MeetingDetails,
            opentalk_types_common::events::invites::EmailInviteRole,
            opentalk_types_common::events::invites::EventInviteStatus,
            opentalk_types_common::events::invites::InviteRole,
            opentalk_types_common::features::FeatureId,
            opentalk_types_common::modules::ModuleId,
            opentalk_types_common::order::Ordering,
            opentalk_types_common::rooms::BreakoutRoomId,
            opentalk_types_common::rooms::RoomId,
            opentalk_types_common::rooms::RoomPassword,
            opentalk_types_common::rooms::invite_codes::InviteCode,
            opentalk_types_common::shared_folders::SharedFolder,
            opentalk_types_common::shared_folders::SharedFolderAccess,
            opentalk_types_common::streaming::RoomStreamingTarget,
            opentalk_types_common::streaming::RoomStreamingTargetResource,
            opentalk_types_common::streaming::StreamingKey,
            opentalk_types_common::streaming::StreamingLink,
            opentalk_types_common::streaming::StreamingTarget,
            opentalk_types_common::streaming::StreamingTargetResource,
            opentalk_types_common::streaming::StreamingTargetId,
            opentalk_types_common::streaming::StreamingTargetKind,
            opentalk_types_common::streaming::StreamingTargetKindResource,
            opentalk_types_common::tariffs::TariffId,
            opentalk_types_common::tariffs::TariffModuleResource,
            opentalk_types_common::tariffs::TariffResource,
            opentalk_types_common::tariffs::TariffStatus,
            opentalk_types_common::time::DateTimeTz,
            opentalk_types_common::time::RecurrencePattern,
            opentalk_types_common::time::RecurrenceRule,
            opentalk_types_common::time::TimeZone,
            opentalk_types_common::time::Timestamp,
            opentalk_types_common::users::DisplayName,
            opentalk_types_common::users::Language,
            opentalk_types_common::users::Theme,
            opentalk_types_common::users::UserId,
            opentalk_types_common::users::UserTitle,
        ),
        responses(
            crate::api::responses::BadRequest,
            crate::api::responses::BinaryData,
            crate::api::responses::InternalServerError,
            crate::api::responses::Unauthorized,
            crate::api::responses::Forbidden,
            crate::api::responses::NotFound,
        ),
    ),
    modifiers(&SecurityAddon),
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};

        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "BearerAuth",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
        // TODO: this is strictly speaking no bearer authentication, so we
        // need to find out whether we can properly describe what we implemented with
        // the `Authorization: InviteCode …` header.
        // Supported authentication schemes:
        // https://www.iana.org/assignments/http-authschemes/http-authschemes.xhtml
        components.add_security_scheme(
            "InviteCode",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
    }
}

fn v1_scope(
    settings_provider: SettingsProvider,
    authz: Data<kustos::Authz>,
    db: Data<Db>,
    oidc_ctx: Data<OidcContext>,
    acl: kustos::actix_web::KustosService,
) -> Scope {
    // the latest version contains the root services

    let scope = web::scope("/v1");

    scope
        .service(api::v1::auth::post_login)
        .service(api::v1::auth::get_login)
        .service(api::v1::rooms::start_invited)
        .service(api::v1::rooms::roomserver::start_invited)
        .service(api::v1::invites::verify_invite_code)
        .service(api::v1::turn::get)
        .service(
            web::scope("/services")
                .wrap(api::v1::middleware::service_auth::ServiceAuth::new(
                    oidc_ctx.clone(),
                ))
                .service(api::v1::services::call_in::services())
                .service(api::v1::services::recording::services()),
        )
        .service(
            // empty scope to differentiate between auth endpoints
            web::scope("")
                .wrap(acl)
                .wrap(api::v1::middleware::user_auth::OidcAuth {
                    settings_provider,
                    authz,
                    db,
                    oidc_ctx,
                })
                .service(api::v1::users::find)
                .service(api::v1::users::patch_me)
                .service(api::v1::users::get_me)
                .service(api::v1::users::get_me_tariff)
                .service(api::v1::users::get_me_assets)
                .service(api::v1::users::get_user)
                .service(api::v1::rooms::accessible)
                .service(api::v1::rooms::new)
                .service(api::v1::rooms::patch)
                .service(api::v1::rooms::get)
                .service(api::v1::rooms::get_room_event)
                .service(api::v1::rooms::get_room_tariff)
                .service(api::v1::rooms::start)
                .service(api::v1::rooms::roomserver::start)
                .service(api::v1::rooms::delete)
                .service(api::v1::events::new_event)
                .service(api::v1::events::get_events)
                .service(api::v1::events::get_event)
                .service(api::v1::events::patch_event)
                .service(api::v1::events::delete_event)
                .service(api::v1::events::favorites::add_event_to_favorites)
                .service(api::v1::events::favorites::remove_event_from_favorites)
                .service(api::v1::events::instances::get_event_instance)
                .service(api::v1::events::instances::get_event_instances)
                .service(api::v1::events::instances::patch_event_instance)
                .service(api::v1::events::invites::create_invite_to_event)
                .service(api::v1::events::invites::get_invites_for_event)
                .service(api::v1::events::invites::delete_email_invite_to_event)
                .service(api::v1::events::invites::delete_invite_to_event)
                .service(api::v1::events::invites::update_email_invite_to_event)
                .service(api::v1::events::invites::update_invite_to_event)
                .service(api::v1::events::invites::accept_event_invite)
                .service(api::v1::events::invites::decline_event_invite)
                .service(api::v1::events::shared_folder::get_shared_folder_for_event)
                .service(api::v1::events::shared_folder::put_shared_folder_for_event)
                .service(api::v1::events::shared_folder::delete_shared_folder_for_event)
                .service(api::v1::sip_configs::get)
                .service(api::v1::sip_configs::put)
                .service(api::v1::sip_configs::delete)
                .service(api::v1::invites::get_invites)
                .service(api::v1::invites::add_invite)
                .service(api::v1::invites::get_invite)
                .service(api::v1::invites::update_invite)
                .service(api::v1::invites::delete_invite)
                .service(api::v1::assets::room_assets)
                .service(api::v1::assets::room_asset)
                .service(api::v1::assets::create)
                .service(api::v1::assets::delete)
                .service(api::v1::streaming_targets::get_streaming_targets)
                .service(api::v1::streaming_targets::post_streaming_target)
                .service(api::v1::streaming_targets::get_streaming_target)
                .service(api::v1::streaming_targets::patch_streaming_target)
                .service(api::v1::streaming_targets::delete_streaming_target),
        )
}

fn setup_cors() -> Cors {
    use actix_web::http::{Method, header::*};

    // Use a permissive CORS configuration.
    // The HTTP API is using Bearer tokens for authentication, which are handled by the application and not the browser.
    Cors::default()
        .allow_any_origin()
        .send_wildcard()
        .allowed_header(CONTENT_TYPE)
        .allowed_header(AUTHORIZATION)
        .allowed_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
}

/// Set up TLS for the HTTP server that is provided by the controller
///
/// Receives the TLS-related settings from the controller configuration
/// which contains the path to the private key and the certificate files
/// from where the TLS configuration is loaded and set up.
fn setup_rustls(tls: &HttpTls) -> Result<rustls::ServerConfig> {
    let cert_file = File::open(&tls.certificate).with_whatever_context(|_| {
        format!("Failed to open certificate file {:?}", &tls.certificate)
    })?;
    let certs = rustls_pemfile::certs(&mut BufReader::new(cert_file))
        .collect::<Result<Vec<CertificateDer>, _>>()
        .whatever_context("Invalid certificate")?;

    let private_key_file = File::open(&tls.private_key).with_whatever_context(|_| {
        format!(
            "Failed to open pkcs8 private key file {:?}",
            &tls.private_key
        )
    })?;
    let mut key = rustls_pemfile::pkcs8_private_keys(&mut BufReader::new(private_key_file))
        .collect::<Result<Vec<PrivatePkcs8KeyDer>, _>>()
        .whatever_context("Invalid pkcs8 private key")?;

    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, rustls_pki_types::PrivateKeyDer::Pkcs8(key.remove(0)))
        .whatever_context("Invalid DER-encoded key ")?;

    Ok(config)
}

struct ModuleInitializer {
    init_data: SignalingModuleInitData,
    signaling_modules: SignalingModules,
}

#[async_trait(?Send)]
impl ModulesRegistrar for ModuleInitializer {
    type Error = Whatever;

    async fn register<M: SignalingModule>(&mut self) -> Result<()> {
        let params = M::build_params(self.init_data.clone())
            .await
            .with_whatever_context(|_| format!("Failed to initialize module '{}'", M::NAMESPACE))?;

        if let Some(params) = params {
            self.signaling_modules.add_module::<M>(params);
        } else {
            log::info!(
                "Skipping module '{}' due to missing configuration",
                M::NAMESPACE
            );
        }

        Ok(())
    }
}

fn is_ipv6_available() -> bool {
    TcpListener::bind((Ipv6Addr::UNSPECIFIED, 0)).is_ok()
}

fn determine_socket_address(
    config_address: Option<&str>,
    config_port: u16,
) -> std::io::Result<Vec<SocketAddr>> {
    let to_socket_addrs = if let Some(addr) = config_address {
        Vec::from_iter((addr, config_port).to_socket_addrs()?)
    } else if is_ipv6_available() {
        Vec::from_iter((Ipv6Addr::UNSPECIFIED, config_port).to_socket_addrs()?)
    } else {
        Vec::from_iter((Ipv4Addr::UNSPECIFIED, config_port).to_socket_addrs()?)
    };
    Ok(to_socket_addrs)
}
