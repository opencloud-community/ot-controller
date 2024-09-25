// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::BTreeSet, fmt::Debug, sync::Arc};

use lapin_pool::RabbitMqPool;
use opentalk_controller_settings::{Settings, SharedSettings};
use opentalk_types::api::error::ApiError;
use opentalk_types_common::{features::FeatureId, modules::ModuleId};
use opentalk_types_signaling::{SignalingModuleFrontendData, SignalingModulePeerFrontendData};
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use tokio::sync::broadcast;

use crate::{
    room_lock::LockError, DestroyContext, Event, InitContext, ModuleContext, VolatileStorage,
};

type Result<T> = std::result::Result<T, SignalingModuleError>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
#[non_exhaustive]
pub enum SignalingModuleError {
    #[snafu(display("Redis error: {message}",))]
    RedisError {
        message: String,
        source: redis::RedisError,
    },

    #[snafu(context(false))]
    UrlParseError { source: url::ParseError },

    #[snafu(context(false))]
    UuidError { source: uuid::Error },

    #[snafu(context(false))]
    ReqwestError { source: reqwest::Error },

    #[snafu(display("NotFound: {message}",))]
    NotFoundError { message: String },

    #[snafu(context(false))]
    R3dlockError { source: opentalk_r3dlock::Error },

    #[snafu(context(false))]
    RoomLockError { source: LockError },

    #[cfg(feature = "module_tester")]
    #[snafu(transparent)]
    NoInitError {
        source: crate::module_tester::NoInitError,
    },

    #[snafu(context(false))]
    DatabaseError {
        source: opentalk_database::DatabaseError,
    },

    #[snafu(context(false))]
    LapinError { source: lapin::Error },

    #[snafu(context(false))]
    LapinPoolError { source: lapin_pool::Error },

    #[snafu(context(false))]
    EtherpadError {
        source: opentalk_etherpad_client::EtherpadError,
    },

    #[snafu(display("Failed to deserialize config",))]
    ConfigError { source: config::ConfigError },

    #[snafu(display("SerdeJson error: {message}",))]
    SerdeJsonError {
        message: String,
        source: serde_json::Error,
    },

    #[snafu(display("Janus error: {message}"))]
    JanusClientError {
        message: String,
        source: opentalk_janus_client::error::Error,
    },

    #[snafu(display("Custom error: {message}"), whatever)]
    CustomError {
        message: String,

        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, Some)))]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl From<SignalingModuleError> for ApiError {
    fn from(value: SignalingModuleError) -> Self {
        log::error!("SignalingModule error: {value}");
        ApiError::internal()
    }
}

#[derive(Clone)]
pub struct SignalingModuleInitData {
    pub startup_settings: Arc<Settings>,
    pub shared_settings: SharedSettings,
    pub rabbitmq_pool: Arc<RabbitMqPool>,
    pub volatile: VolatileStorage,
    pub shutdown: broadcast::Sender<()>,
    pub reload: broadcast::Sender<()>,
}

/// Extension to a the signaling websocket
#[async_trait::async_trait(?Send)]
pub trait SignalingModule: Send + Sized + 'static {
    /// Defines the websocket message namespace
    ///
    /// Must be unique between all registered modules.
    const NAMESPACE: &'static str;

    /// The module params, can be any type that is `Clone` + `Send` + `Sync`
    ///
    /// Will get passed to `init` as parameter
    type Params: Clone + Send + Sync;

    /// The websocket incoming message type
    type Incoming: for<'de> Deserialize<'de>;

    /// The websocket outgoing message type
    type Outgoing: Serialize + PartialEq + Debug;

    /// Message type sent over the message exchange to other participant's modules
    type ExchangeMessage: for<'de> Deserialize<'de> + Serialize;

    /// Optional event type, yielded by `ExtEventStream`
    ///
    /// If the module does not register external events it should be set to `()`.
    type ExtEvent;

    /// Data about the owning user of the ws-module which is sent to the frontend on join
    type FrontendData: SignalingModuleFrontendData;

    /// Data about a peer which is sent to the frontend
    type PeerFrontendData: SignalingModulePeerFrontendData;

    /// Constructor of the module
    ///
    /// Provided with the websocket context the modules params and the negotiated protocol
    /// The module can decide to no initiate based on the protocol and passed ctx and params.
    /// E.g. when the user is a bot or guest.
    async fn init(
        ctx: InitContext<'_, Self>,
        params: &Self::Params,
        protocol: &'static str,
    ) -> Result<Option<Self>>;

    /// Returns the features provided by a particular module.
    fn get_provided_features() -> BTreeSet<FeatureId> {
        BTreeSet::default()
    }

    /// Events related to this module will be passed into this function together with [`ModuleContext`]
    /// which gives access to the websocket and other related information.
    async fn on_event(
        &mut self,
        ctx: ModuleContext<'_, Self>,
        event: Event<'_, Self>,
    ) -> Result<()>;

    /// Before dropping the module this function will be called
    async fn on_destroy(self, ctx: DestroyContext<'_>);

    /// Build the parameters for instantiating the signaling module.
    ///
    /// If `None` is returned, the module is not initialized.
    async fn build_params(init: SignalingModuleInitData) -> Result<Option<Self::Params>>;

    fn module_id() -> ModuleId {
        Self::NAMESPACE
            .parse()
            .expect("namespace must be a valid module id")
    }
}
