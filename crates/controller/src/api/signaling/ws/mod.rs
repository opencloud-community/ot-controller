// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::metrics::SignalingMetrics;
use super::prelude::*;
use crate::api::signaling::ws_modules::control::ControlData;
use crate::api::signaling::{Role, SignalingRoomId};
use crate::api::Participant;
use crate::redis_wrapper::RedisConnection;
use crate::storage::ObjectStorage;
use actix_http::ws::CloseCode;
use anyhow::Result;
use database::Db;
use db_storage::rooms::Room;
use db_storage::users::User;
use futures::stream::SelectAll;
use kustos::Authz;
use modules::{any_stream, AnyStream};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio_stream::Stream;
use types::{
    core::{BreakoutRoomId, ParticipantId, Timestamp},
    signaling::{NamespacedCommand, NamespacedEvent},
};

mod actor;
mod echo;
mod http;
pub mod module_tester;
mod modules;
mod runner;

pub use echo::Echo;
pub use http::ws_service;
pub use http::SignalingModules;
pub use http::SignalingProtocols;

/// Event passed to [`SignalingModule::on_event`]
pub enum Event<'evt, M>
where
    M: SignalingModule,
{
    /// The participant joined the room
    Joined {
        /// Data set by the control module. Some modules require attributes specified by the
        /// control module which are provided here on join
        control_data: &'evt ControlData,

        /// The module can set this option to Some(M::FrontendData) to populate
        /// the `join_success` message with additional information to the frontend module counterpart
        frontend_data: &'evt mut Option<M::FrontendData>,

        /// List of participants already inside the room.
        ///
        /// The module can populate participant specific frontend-data, which is sent inside
        /// the participant inside the `join_success` message
        participants: &'evt mut HashMap<ParticipantId, Option<M::PeerFrontendData>>,
    },

    /// The participant is in the process of leaving the room, this event will be called before
    /// `on_destroy` is called and before the exchange control message `Left` has been sent.
    ///
    /// Note: Calls to `ModuleContext::ws_send` when receiving this event will almost certainly fail
    Leaving,

    /// A user can request attention by 'raising' his hand, this event gets broadcast to every
    /// module.
    RaiseHand,

    /// User lowered his hand and no longer requests attention.
    LowerHand,

    /// Participant with the associated id has joined the room
    ParticipantJoined(ParticipantId, &'evt mut Option<M::PeerFrontendData>),

    /// Participant with the associated id has left the room
    ParticipantLeft(ParticipantId),

    /// Participant data has changed, an options to `M::PeerFrontendData`
    ParticipantUpdated(ParticipantId, &'evt mut Option<M::PeerFrontendData>),

    /// Received websocket message
    WsMessage(M::Incoming),

    /// Exchange subscriber received a message for this module
    Exchange(M::ExchangeMessage),

    /// External event provided by eventstream which was added using [`InitContext::add_event_stream`].
    ///
    /// Modules that didn't register external events will
    /// never receive this variant and can ignore it.
    Ext(M::ExtEvent),
}

/// Context passed to the `init` function
pub struct InitContext<'ctx, M>
where
    M: SignalingModule,
{
    id: ParticipantId,
    room: &'ctx Room,
    breakout_room: Option<BreakoutRoomId>,
    participant: &'ctx Participant<User>,
    role: Role,
    db: &'ctx Arc<Db>,
    storage: &'ctx Arc<ObjectStorage>,
    authz: &'ctx Arc<Authz>,
    exchange_bindings: &'ctx mut Vec<ExchangeBinding>,
    events: &'ctx mut SelectAll<AnyStream>,
    redis_conn: &'ctx mut RedisConnection,
    m: PhantomData<fn() -> M>,
}

struct ExchangeBinding {
    routing_key: String,
}

impl<M> InitContext<'_, M>
where
    M: SignalingModule,
{
    /// ID of the participant the module instance belongs to
    pub fn participant_id(&self) -> ParticipantId {
        self.id
    }

    /// Returns a reference to the database representation of the the room
    ///
    /// Note that the room will always be the same regardless if inside a
    /// breakout room or not.
    pub fn room(&self) -> &Room {
        self.room
    }

    /// ID of the room currently inside, this MUST be used when a module does not care about
    /// whether it is inside a breakout room or not.
    pub fn room_id(&self) -> SignalingRoomId {
        SignalingRoomId(self.room.id, self.breakout_room)
    }

    /// Returns the ID of the breakout room, if inside one
    pub fn breakout_room(&self) -> Option<BreakoutRoomId> {
        self.breakout_room
    }

    /// Returns the user associated with the participant
    pub fn participant(&self) -> &Participant<User> {
        self.participant
    }

    /// Returns the role of participant inside the room
    pub fn role(&self) -> Role {
        self.role
    }

    /// Returns a reference to the controllers database interface
    pub fn db(&self) -> &Arc<Db> {
        self.db
    }

    /// Returns a reference to the controllers S3 storage interface
    pub fn storage(&self) -> &Arc<ObjectStorage> {
        self.storage
    }

    pub fn authz(&self) -> &Arc<Authz> {
        self.authz
    }

    /// Access to a redis connection
    pub fn redis_conn(&mut self) -> &mut RedisConnection {
        self.redis_conn
    }

    /// Add a routing-key for the exchange-subscriber to bind to
    pub fn add_exchange_binding(&mut self, routing_key: String) {
        self.exchange_bindings.push(ExchangeBinding { routing_key });
    }

    /// Add a custom event stream which return `M::ExtEvent`
    pub fn add_event_stream<S>(&mut self, stream: S)
    where
        S: Stream<Item = M::ExtEvent> + 'static,
    {
        self.events.push(any_stream(M::NAMESPACE, stream));
    }
}

/// Context passed to the module
///
/// Can be used to send websocket messages
pub struct ModuleContext<'ctx, M>
where
    M: SignalingModule,
{
    role: Role,
    ws_messages: &'ctx mut Vec<NamespacedEvent<'static, M::Outgoing>>,
    timestamp: Timestamp,
    exchange_publish: &'ctx mut Vec<ExchangePublish>,
    redis_conn: &'ctx mut RedisConnection,
    events: &'ctx mut SelectAll<AnyStream>,
    invalidate_data: &'ctx mut bool,
    exit: &'ctx mut Option<CloseCode>,
    metrics: Option<Arc<SignalingMetrics>>,
    m: PhantomData<fn() -> M>,
}

#[derive(Debug, Clone)]
struct ExchangePublish {
    routing_key: String,
    message: String,
}

impl<M> ModuleContext<'_, M>
where
    M: SignalingModule,
{
    pub fn role(&self) -> Role {
        self.role
    }

    /// Queue a outgoing message to be sent via the websocket
    /// after exiting the `on_event` function
    pub fn ws_send(&mut self, message: M::Outgoing) {
        self.ws_send_overwrite_timestamp(message, self.timestamp)
    }

    /// Similar to `ws_send` but sets an explicit timestamp
    pub fn ws_send_overwrite_timestamp(&mut self, message: M::Outgoing, timestamp: Timestamp) {
        self.ws_messages.push(NamespacedEvent {
            namespace: M::NAMESPACE,
            timestamp,
            payload: message,
        });
    }

    /// Queue a outgoing message to be sent via the message exchange
    pub fn exchange_publish(&mut self, routing_key: String, message: M::ExchangeMessage) {
        self.exchange_publish_any(
            routing_key,
            NamespacedEvent {
                namespace: M::NAMESPACE,
                timestamp: self.timestamp,
                payload: message,
            },
        );
    }

    /// Queue any serializable outgoing message to be sent via the message exchange
    pub fn exchange_publish_any(&mut self, routing_key: String, message: impl Serialize) {
        self.exchange_publish.push(ExchangePublish {
            routing_key,
            message: serde_json::to_string(&message).expect("value must be serializable to json"),
        });
    }

    /// Queue a outgoing control message
    ///
    /// Used in modules which control some behavior in the control module/runner
    pub(crate) fn exchange_publish_control(
        &mut self,
        routing_key: String,
        message: control::exchange::Message,
    ) {
        self.exchange_publish_any(
            routing_key,
            NamespacedEvent {
                namespace: control::NAMESPACE,
                timestamp: self.timestamp,
                payload: message,
            },
        );
    }

    /// Access to the storage of the room
    pub fn redis_conn(&mut self) -> &mut RedisConnection {
        self.redis_conn
    }

    /// Add a custom event stream which return `M::ExtEvent`
    pub fn add_event_stream<S>(&mut self, stream: S)
    where
        S: Stream<Item = M::ExtEvent> + 'static,
    {
        self.events.push(any_stream(M::NAMESPACE, stream));
    }

    /// Signals that the data related to the participant has changed
    pub fn invalidate_data(&mut self) {
        *self.invalidate_data = true;
    }

    pub fn exit(&mut self, code: Option<CloseCode>) {
        *self.exit = Some(code.unwrap_or(CloseCode::Normal));
    }

    pub fn metrics(&self) -> Option<&Arc<SignalingMetrics>> {
        self.metrics.as_ref()
    }

    /// Returns the Timestamp of the event which triggered the `on_event` handler.
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
}

/// Context passed to the `destroy` function
pub struct DestroyContext<'ctx> {
    redis_conn: &'ctx mut RedisConnection,
    destroy_room: bool,
}

impl DestroyContext<'_> {
    /// Access to a redis connection
    pub fn redis_conn(&mut self) -> &mut RedisConnection {
        self.redis_conn
    }

    /// Returns true if the module belongs to the last participant inside a room
    pub fn destroy_room(&self) -> bool {
        self.destroy_room
    }
}

/// Extension to a the signaling websocket
#[async_trait::async_trait(?Send)]
pub trait SignalingModule: Sized + 'static {
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
    type FrontendData: Serialize;

    /// Data about a peer which is sent to the frontend
    type PeerFrontendData: Serialize;

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

    /// Events related to this module will be passed into this function together with [`ModuleContext`]
    /// which gives access to the websocket and other related information.
    async fn on_event(
        &mut self,
        ctx: ModuleContext<'_, Self>,
        event: Event<'_, Self>,
    ) -> Result<()>;

    /// Before dropping the module this function will be called
    async fn on_destroy(self, ctx: DestroyContext<'_>);
}
