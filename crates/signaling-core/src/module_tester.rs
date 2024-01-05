// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! The ModuleTester simulates a runner environment for a specified module.
//!
//! This module is exclusively used for testing and does not contribute to the controllers behavior.
//! As its basically a 'copy' of the Runner it uses a few types from there. Due to
//! visibility restriction of those types, this module is located in the same folder.
//!
//! The idea is to simulate a frontend websocket connection.
use crate::{
    control::{self, storage, ControlStateExt as _},
    AnyStream, DestroyContext, Event, ExchangePublish, InitContext, ModuleContext, ObjectStorage,
    Participant, RedisConnection, SignalingModule, SignalingRoomId,
};
use actix_http::ws::CloseCode;
use actix_rt::task::JoinHandle;
use anyhow::{bail, Context, Result};
use database::Db;
use db_storage::rooms::Room;
use db_storage::users::User;
use futures::stream::SelectAll;
use futures::StreamExt;
use kustos::Authz;
use serde_json::Value;
use tokio::{
    select,
    sync::{
        broadcast,
        mpsc::{self, UnboundedReceiver, UnboundedSender},
    },
    task,
    time::{timeout, timeout_at, Instant},
};
use types::{
    common::tariff::TariffResource,
    core::{BreakoutRoomId, ParticipantId, ParticipationKind, TariffId, Timestamp, UserId},
    signaling::{
        control::{
            command::{ControlCommand, Join},
            event::{ControlEvent, JoinSuccess},
            state::ControlState,
            AssociatedParticipant, NAMESPACE,
        },
        ModuleData, NamespacedCommand, NamespacedEvent, Role,
    },
};

use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::panic;
use std::sync::Arc;
use std::time::Duration;

/// A module tester that simulates a runner environment for provided module.
///
/// When created, the `ModuleTester` instance acts like a client websocket connection. This means
/// that incoming events like `Join`, `RaiseHand` and `LowerHand` can be sent to the underlying module as well
/// as module specific WebSocket messages. Outgoing messages like `JoinSuccess`, `Joined`, `Left`, etc. can
/// be received via an internal channel. See [`ModuleTester::send_ws_message`] & [`ModuleTester::receive_ws_message`]
/// for more details.
pub struct ModuleTester<M>
where
    M: SignalingModule,
{
    /// The redis interface
    pub redis_conn: RedisConnection,
    /// The database interface
    pub db: Arc<Db>,
    /// Authz
    pub authz: Arc<Authz>,
    /// The room that the users are inside
    room: Room,
    /// Optional breakout room id
    breakout_room: Option<BreakoutRoomId>,

    /// A map of RunnerInterfaces with their JoinHandle, each for a participant
    runner_interfaces: HashMap<ParticipantId, (RunnerInterface<M>, JoinHandle<()>)>,
    /// A broadcast channel that mocks a the message exchange
    exchange_sender: broadcast::Sender<ExchangePublish>,
}

impl<M> ModuleTester<M>
where
    M: SignalingModule,
{
    /// Create a new ModuleTester instance
    pub fn new(db: Arc<Db>, authz: Arc<Authz>, redis_conn: RedisConnection, room: Room) -> Self {
        let (exchange_sender, _) = broadcast::channel(10);

        Self {
            redis_conn,
            db,
            authz,
            room,
            // todo: add breakout room support
            breakout_room: None,
            runner_interfaces: HashMap::new(),
            exchange_sender,
        }
    }

    async fn join_internal(
        &mut self,
        participant_id: ParticipantId,
        participant: Participant<User>,
        role: Role,
        display_name: &str,
        params: M::Params,
    ) -> Result<()> {
        let (client_interface, runner_interface) = create_interfaces::<M>().await;

        let runner = MockRunner::<M>::new(
            participant_id,
            self.room.clone(),
            self.breakout_room,
            participant.clone(),
            role,
            self.db.clone(),
            Arc::new(ObjectStorage::broken()),
            self.authz.clone(),
            self.redis_conn.clone(),
            params,
            client_interface,
            self.exchange_sender.clone(),
        )
        .await?;

        let runner_handle = task::spawn_local(runner.run());

        runner_interface
            .ws
            .send(WsMessageIncoming::Control(ControlCommand::Join(Join {
                display_name: Some(display_name.into()),
            })))?;

        self.runner_interfaces
            .insert(participant_id, (runner_interface, runner_handle));

        Ok(())
    }

    /// Join the ModuleTester as the specified user
    ///
    /// This is the equivalent of joining a room in the real controller. Spawns a underlying runner task that
    /// can send and receive WebSocket messages.
    pub async fn join_user(
        &mut self,
        participant_id: ParticipantId,
        user: User,
        role: Role,
        display_name: &str,
        params: M::Params,
    ) -> Result<()> {
        self.join_internal(
            participant_id,
            Participant::User(user),
            role,
            display_name,
            params,
        )
        .await?;
        Ok(())
    }

    /// Join the ModuleTester as the specified user
    ///
    /// This is the equivalent of joining a room in the real controller. Spawns a underlying runner task that
    /// can send and receive WebSocket messages.
    pub async fn join_guest(
        &mut self,
        participant_id: ParticipantId,
        display_name: &str,
        params: M::Params,
    ) -> Result<()> {
        self.join_internal(
            participant_id,
            Participant::Guest,
            Role::Guest,
            display_name,
            params,
        )
        .await?;

        Ok(())
    }

    /// Send a module specific WebSocket message to the underlying module that is mapped to `participant_id`.
    ///
    /// # Note
    /// WebSocket control messages (e.g. [`RaiseHand`](ControlCommand::RaiseHand),
    /// [`LowerHand`](ControlCommand::LowerHand)) have to be sent via their respective helper function.
    pub fn send_ws_message(
        &self,
        participant_id: &ParticipantId,
        message: M::Incoming,
    ) -> Result<()> {
        let (interface, ..) = self
            .runner_interfaces
            .get(participant_id)
            .expect("User {} does not exist in module tester");

        interface.ws.send(WsMessageIncoming::Module(message))?;

        Ok(())
    }

    /// Receive a WebSocket message from the underlying Module that is mapped to `participant_id`
    ///
    ///
    /// This function will yield when there is no available message and timeout after two seconds.
    /// When a longer timeout or deadline is required, use
    /// [`ModuleTester::receive_ws_message_override_timeout`] or
    /// [`ModuleTester::receive_ws_message_override_timeout_at`].
    ///
    /// # Returns
    /// - Ok([`WsMessageOutgoing`]) when a message is available within the timeout window.
    /// - Err([`anyhow::Error`]) on timeout or when the internal channel has been closed.
    pub async fn receive_ws_message(
        &mut self,
        participant_id: &ParticipantId,
    ) -> Result<WsMessageOutgoing<M>> {
        self.receive_ws_message_override_timeout(participant_id, Duration::from_secs(2))
            .await
    }

    /// Receive a WebSocket message from the underlying Module that is mapped to `participant_id`
    ///
    /// Behaves like [`ModuleTester::receive_ws_message`] but allows a custom timeout.
    pub async fn receive_ws_message_override_timeout(
        &mut self,
        participant_id: &ParticipantId,
        timeout_duration: Duration,
    ) -> Result<WsMessageOutgoing<M>> {
        let interface = self.get_runner_interface(participant_id)?;

        match timeout(timeout_duration, interface.ws.recv()).await? {
            Some(message) => Ok(message),
            None => bail!("Failed to receive ws message in module tester"),
        }
    }

    /// Receive a WebSocket message from the underlying Module that is mapped to `participant_id`
    ///
    /// Behaves like [`ModuleTester::receive_ws_message`] but allows a custom deadline.
    pub async fn receive_ws_message_override_timeout_at(
        &mut self,
        participant_id: &ParticipantId,
        deadline: Instant,
    ) -> Result<WsMessageOutgoing<M>> {
        let interface = self.get_runner_interface(participant_id)?;

        match timeout_at(deadline, interface.ws.recv()).await? {
            Some(message) => Ok(message),
            None => bail!("Failed to receive ws message in module tester"),
        }
    }

    /// Send a [`RaiseHand`](ControlCommand::RaiseHand) control message to the module/runner.
    pub fn raise_hand(&mut self, participant_id: &ParticipantId) -> Result<()> {
        let interface = self.get_runner_interface(participant_id)?;
        interface
            .ws
            .send(WsMessageIncoming::Control(ControlCommand::RaiseHand))
    }

    /// Send a [`LowerHand`](ControlCommand::LowerHand) control message to the module/runner.
    pub fn lower_hand(&mut self, participant_id: &ParticipantId) -> Result<()> {
        let interface = self.get_runner_interface(participant_id)?;

        interface
            .ws
            .send(WsMessageIncoming::Control(ControlCommand::LowerHand))
    }

    /// Close the WebSocket channel and leave the room with the participant
    ///
    /// # Panics
    /// When the participants runner panicked
    pub async fn leave(&mut self, participant_id: &ParticipantId) -> Result<()> {
        let (interface, handle) = self.get_runner(participant_id)?;

        interface.ws.send(WsMessageIncoming::CloseWs)?;

        // expect the runner to shutdown within 3 seconds
        match timeout(Duration::from_secs(3), handle)
            .await
            .context("Failed to shutdown MockRunner within 3 seconds after leave event")?
        {
            Ok(_) => {
                self.runner_interfaces.remove(participant_id);
                Ok(())
            }
            Err(join_error) => {
                if join_error.is_panic() {
                    panic::resume_unwind(join_error.into_panic());
                }

                bail!(join_error);
            }
        }
    }

    /// Get the [`RunnerInterface`] of the runner that is mapped to `participant_id`
    fn get_runner_interface(
        &mut self,
        participant_id: &ParticipantId,
    ) -> Result<&mut RunnerInterface<M>> {
        Ok(&mut self.get_runner(participant_id)?.0)
    }

    /// Get the [`RunnerInterface`] & [`JoinHandle`] of the runner that is mapped to `participant_id`
    fn get_runner(
        &mut self,
        participant_id: &ParticipantId,
    ) -> Result<&mut (RunnerInterface<M>, JoinHandle<()>)> {
        self.runner_interfaces
            .get_mut(participant_id)
            .with_context(|| {
                format!("Participant {participant_id} does not exist in module tester",)
            })
    }

    fn get_participants(&self) -> Vec<ParticipantId> {
        self.runner_interfaces
            .iter()
            .map(|(participant, ..)| *participant)
            .collect()
    }

    /// Shutdown the ModuleTester
    ///
    /// Leave the room with all participants. Continues to unwind panics that happened in any runner.
    pub async fn shutdown(mut self) -> Result<()> {
        let participants = self.get_participants();

        for participant_id in participants {
            self.leave(&participant_id).await?;
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Module did not initialize")]
pub struct NoInitError;

/// Acts like a [Runner](super::runner::Runner) for a single specific module.
struct MockRunner<M>
where
    M: SignalingModule,
{
    redis_conn: RedisConnection,
    room_id: SignalingRoomId,
    room_owner: UserId,
    participant_id: ParticipantId,
    participant: Participant<UserId>,
    role: Role,
    control_data: Option<ControlState>,
    module: M,
    interface: ClientInterface<M>,
    exchange_sender: broadcast::Sender<ExchangePublish>,
    events: SelectAll<AnyStream>,
    exit: bool,
}

#[allow(clippy::too_many_arguments)]
impl<M> MockRunner<M>
where
    M: SignalingModule,
{
    /// Create a new runner and initialize the underlying module.
    async fn new(
        participant_id: ParticipantId,
        mut room: Room,
        breakout_room: Option<BreakoutRoomId>,
        mut participant: Participant<User>,
        role: Role,
        db: Arc<Db>,
        storage: Arc<ObjectStorage>,
        authz: Arc<Authz>,
        mut redis_conn: RedisConnection,
        params: M::Params,
        interface: ClientInterface<M>,
        exchange_sender: broadcast::Sender<ExchangePublish>,
    ) -> Result<Self> {
        let mut events = SelectAll::new();

        let init_context = InitContext {
            id: participant_id,
            room: &mut room,
            breakout_room,
            participant: &mut participant,
            role,
            db: &db,
            storage: &storage,
            authz: &authz,
            exchange_bindings: &mut vec![],
            events: &mut events,
            redis_conn: &mut redis_conn,
            m: PhantomData::<fn() -> M>,
        };

        let module = M::init(init_context, &params, "")
            .await
            .expect("Module failed to initialize with the passed parameters")
            .ok_or(NoInitError)?;
        let participant = match participant {
            Participant::User(user) => Participant::User(user.id),
            Participant::Guest => Participant::Guest,
            Participant::Sip => Participant::Sip,
            Participant::Recorder => Participant::Recorder,
        };

        Ok(Self {
            redis_conn,
            room_id: SignalingRoomId::new(room.id, breakout_room),
            room_owner: room.created_by,
            participant_id,
            participant,
            role,
            control_data: Option::<ControlState>::None,
            module,
            interface,
            exchange_sender,
            events,
            exit: false,
        })
    }

    /// The MockRunners event loop
    async fn run(mut self) {
        let mut exchange_receiver = self.exchange_sender.subscribe();

        while !self.exit {
            let mut ws_messages = vec![];
            let mut exchange_publish = vec![];
            let mut invalidate_data = false;
            let mut events = SelectAll::new();
            let mut exit = None;

            let ctx = ModuleContext {
                role: self.role,
                timestamp: Timestamp::now(),
                ws_messages: &mut ws_messages,
                exchange_publish: &mut exchange_publish,
                redis_conn: &mut self.redis_conn.clone(),
                invalidate_data: &mut invalidate_data,
                events: &mut events,
                exit: &mut exit,
                metrics: None,
                m: PhantomData::<fn() -> M>,
            };

            select! {
                res = self.interface.ws.recv() => {
                    let ws_message = res.expect("MockRunners websocket channel is broken");

                    match ws_message {
                        WsMessageIncoming::Module(module_message) =>
                            self.module.on_event(ctx, Event::WsMessage(module_message)).await.expect("Error when handling incoming ws message"),

                        WsMessageIncoming::Control(control_message) =>
                            self.handle_ws_control_message(ctx, control_message).await.expect("Error when handling incoming ws control message"),

                        WsMessageIncoming::CloseWs => {
                            self.exit = true;
                        },
                    }
                    self.handle_module_requested_actions(ws_messages, exchange_publish, invalidate_data, events, exit).await;
                }
                res = exchange_receiver.recv() => {
                    let message = res.expect("Error when receiving on exchange broadcast channel");

                    self.handle_exchange_message(ctx, message).await.expect("Error when handling exchange message");

                    self.handle_module_requested_actions(ws_messages, exchange_publish, invalidate_data, events, exit).await;
                }
                Some((namespace, message)) = self.events.next() => {
                    assert_eq!(namespace, M::NAMESPACE, "Invalid namespace on external event");

                    self.module.on_event(ctx, Event::Ext(*message.downcast().expect("invalid ext type"))).await.expect("Error when handling external event");

                    self.handle_module_requested_actions(ws_messages, exchange_publish, invalidate_data, events, exit).await;
                }
            }
        }

        log::debug!(
            "Shutting down module for participant {}",
            self.participant_id
        );

        self.leave_room().await.expect("Error while leaving room");

        self.destroy().await.expect("Failed to destroy mock runner");
    }

    async fn handle_ws_control_message(
        &mut self,
        mut ctx: ModuleContext<'_, M>,
        control_message: ControlCommand,
    ) -> Result<()> {
        match control_message {
            ControlCommand::Join(join) => {
                let mut lock = storage::room_mutex(self.room_id);
                let guard = lock.lock(&mut self.redis_conn).await?;

                let is_room_owner =
                    matches!(self.participant, Participant::User(user) if user == self.room_owner);

                let mut attr_pipe = storage::AttrPipeline::new(self.room_id, self.participant_id);

                match &self.participant {
                    Participant::User(user) => {
                        attr_pipe
                            .set("kind", ParticipationKind::User)
                            .set("user_id", user);
                    }
                    Participant::Guest => {
                        attr_pipe.set("kind", ParticipationKind::Guest);
                    }
                    Participant::Sip => {
                        attr_pipe.set("kind", ParticipationKind::Sip);
                    }
                    Participant::Recorder => {
                        attr_pipe.set("kind", ParticipationKind::Recorder);
                    }
                }

                attr_pipe
                    .set("display_name", &join.display_name)
                    .set("role", self.role)
                    .set("avatar_url", &join.display_name)
                    .set("joined_at", ctx.timestamp)
                    .set("hand_is_up", false)
                    .set("hand_updated_at", ctx.timestamp)
                    .set("is_room_owner", is_room_owner)
                    .query_async(&mut self.redis_conn)
                    .await?;

                let participant_set =
                    storage::get_all_participants(&mut self.redis_conn, self.room_id)
                        .await
                        .context("Failed to get all active participants")?;

                storage::add_participant_to_set(
                    &mut self.redis_conn,
                    self.room_id,
                    self.participant_id,
                )
                .await
                .context("Failed to add self to participants set")?;

                guard.unlock(&mut self.redis_conn).await?;

                let mut participants = vec![];

                for id in participant_set {
                    match self.build_participant(id).await {
                        Ok(participant) => participants.push(participant),
                        Err(e) => bail!("Failed to build participant {}, {}", id, e),
                    };
                }

                let mut frontend_data = None;
                let mut participants_data = participants.iter().map(|p| (p.id, None)).collect();

                let avatar_url = match &self.participant {
                    Participant::User(user) => Some(format!("{}{}", "http://example.org/", user)),
                    _ => None,
                };

                let mut control_data = ControlState {
                    display_name: join.display_name.clone().unwrap(),
                    role: self.role,
                    avatar_url: avatar_url.clone(),
                    participation_kind: match &self.participant {
                        Participant::User(_) => ParticipationKind::User,
                        Participant::Guest => ParticipationKind::Guest,
                        Participant::Sip => ParticipationKind::Sip,
                        Participant::Recorder => ParticipationKind::Recorder,
                    },
                    hand_is_up: false,
                    joined_at: ctx.timestamp,
                    left_at: None,
                    hand_updated_at: ctx.timestamp,
                    is_room_owner,
                };

                self.module
                    .on_event(
                        ctx,
                        Event::Joined {
                            frontend_data: &mut frontend_data,
                            participants: &mut participants_data,
                            control_data: &mut control_data,
                        },
                    )
                    .await?;

                self.control_data = Some(control_data);

                let mut module_data = ModuleData::new();

                if let Some(frontend_data) = frontend_data {
                    module_data
                        .insert(&frontend_data)
                        .context("Failed to convert frontend-data to value")?;
                }

                for participant in participants.iter_mut() {
                    if let Some(data) = participants_data.remove(&participant.id).flatten() {
                        participant
                            .module_data
                            .insert(&data)
                            .context("Failed to convert module peer frontend data to value")?;
                    }
                }

                let join_success = JoinSuccess {
                    id: self.participant_id,
                    display_name: join.display_name.unwrap(),
                    avatar_url,
                    role: self.role,
                    closes_at: None,
                    tariff: TariffResource {
                        id: TariffId::nil(),
                        name: "test".into(),
                        quotas: Default::default(),
                        enabled_modules: HashSet::from([M::NAMESPACE.into()]),
                        disabled_features: Default::default(),
                        modules: Default::default(),
                    }
                    .into(),
                    module_data,
                    participants,
                    event_info: None,
                    is_room_owner,
                };

                self.interface
                    .ws
                    .send(WsMessageOutgoing::Control(ControlEvent::JoinSuccess(
                        join_success,
                    )))?;

                self.publish_exchange_control(control::exchange::Message::Joined(
                    self.participant_id,
                ))?;

                Ok(())
            }
            ControlCommand::EnterRoom => unreachable!(),
            ControlCommand::RaiseHand => {
                storage::AttrPipeline::new(self.room_id, self.participant_id)
                    .set("hand_is_up", true)
                    .set("hand_updated_at", ctx.timestamp)
                    .query_async(&mut self.redis_conn)
                    .await?;

                ctx.invalidate_data();

                self.module.on_event(ctx, Event::RaiseHand).await?;

                Ok(())
            }
            ControlCommand::LowerHand => {
                storage::AttrPipeline::new(self.room_id, self.participant_id)
                    .set("hand_is_up", false)
                    .set("hand_updated_at", ctx.timestamp)
                    .query_async(&mut self.redis_conn)
                    .await?;

                ctx.invalidate_data();

                self.module.on_event(ctx, Event::LowerHand).await?;

                Ok(())
            }
            ControlCommand::GrantModeratorRole(_) => unimplemented!(),
            ControlCommand::RevokeModeratorRole(_) => unimplemented!(),
        }
    }

    async fn handle_exchange_control_message(
        &mut self,
        ctx: ModuleContext<'_, M>,
        control_message: control::exchange::Message,
    ) -> Result<()> {
        match control_message {
            control::exchange::Message::Joined(participant_id) => {
                if self.participant_id == participant_id {
                    return Ok(());
                }

                let mut participant = self.build_participant(participant_id).await?;

                let mut data = None;

                self.module
                    .on_event(ctx, Event::ParticipantJoined(participant.id, &mut data))
                    .await
                    .context("Module error on ParticipantJoined event")?;

                if let Some(data) = data {
                    participant.module_data.insert(&data).context(
                        "Failed to serialize PeerFrontendData for ParticipantJoined event",
                    )?;
                }

                self.interface
                    .ws
                    .send(WsMessageOutgoing::Control(ControlEvent::Joined(
                        participant,
                    )))?;

                Ok(())
            }
            control::exchange::Message::Left(participant_id) => {
                if self.participant_id == participant_id {
                    return Ok(());
                }

                self.module
                    .on_event(ctx, Event::ParticipantLeft(participant_id))
                    .await
                    .context("Module error on ParticipantLeft event")?;

                self.interface
                    .ws
                    .send(WsMessageOutgoing::Control(ControlEvent::Left(
                        AssociatedParticipant { id: participant_id },
                    )))?;

                Ok(())
            }
            control::exchange::Message::Update(participant_id) => {
                if self.participant_id == participant_id {
                    return Ok(());
                }

                let mut participant = self.build_participant(participant_id).await?;

                let mut data = None;

                self.module
                    .on_event(ctx, Event::ParticipantUpdated(participant.id, &mut data))
                    .await
                    .context("Module error on ParticipantUpdated event")?;

                if let Some(data) = data {
                    participant.module_data.insert(&data).context(
                        "Failed to serialize PeerFrontendData for ParticipantUpdated event",
                    )?;
                }

                self.interface
                    .ws
                    .send(WsMessageOutgoing::Control(ControlEvent::Update(
                        participant,
                    )))?;

                Ok(())
            }
            control::exchange::Message::Accepted(_participant_id) => {
                todo!()
            }
            control::exchange::Message::SetModeratorStatus(_) => unimplemented!(),
            control::exchange::Message::ResetRaisedHands { issued_by: _ } => unimplemented!(),
            control::exchange::Message::EnableRaiseHands { issued_by: _ } => unimplemented!(),
            control::exchange::Message::DisableRaiseHands { issued_by: _ } => unimplemented!(),
            control::exchange::Message::RoomDeleted => unimplemented!(),
        }
    }

    fn publish_exchange_control(&mut self, message: control::exchange::Message) -> Result<()> {
        let message = serde_json::to_string(&NamespacedCommand {
            namespace: NAMESPACE,
            payload: message,
        })?;

        let exchange_publish = ExchangePublish {
            routing_key: control::exchange::current_room_all_participants(self.room_id),
            message,
        };

        self.exchange_sender
            .send(exchange_publish)
            .map_err(|e| anyhow::Error::msg(format!("Unable to send exchange_publish, {e}")))?;
        Ok(())
    }

    /// Check if the routing key matches this participant and serialize the exchange message
    async fn handle_exchange_message(
        &mut self,
        ctx: ModuleContext<'_, M>,
        exchange_publish: ExchangePublish,
    ) -> Result<()> {
        let participant_routing_key =
            control::exchange::current_room_by_participant_id(self.room_id, self.participant_id);
        match self.participant {
            Participant::User(user) => {
                let user_routing_key =
                    control::exchange::current_room_by_user_id(self.room_id, user);

                if !(exchange_publish.routing_key
                    == control::exchange::current_room_all_participants(self.room_id)
                    || exchange_publish.routing_key == participant_routing_key
                    || exchange_publish.routing_key == user_routing_key)
                {
                    return Ok(());
                }
            }
            Participant::Guest | Participant::Sip | Participant::Recorder => {
                if !(exchange_publish.routing_key
                    == control::exchange::current_room_all_participants(self.room_id)
                    || exchange_publish.routing_key == participant_routing_key)
                {
                    return Ok(());
                }
            }
        }

        let namespaced =
            serde_json::from_str::<NamespacedCommand<Value>>(&exchange_publish.message)
                .context("Failed to read incoming exchange message")?;

        if namespaced.namespace == NAMESPACE {
            let control_message = serde_json::from_value(namespaced.payload)?;

            self.handle_exchange_control_message(ctx, control_message)
                .await
                .context("Error when handling ws control message")?;

            Ok(())
        } else if namespaced.namespace == M::NAMESPACE {
            let module_message = serde_json::from_value(namespaced.payload)?;

            self.module
                .on_event(ctx, Event::Exchange(module_message))
                .await
                .context("Module error on exchange event")?;

            Ok(())
        } else {
            bail!(
                "Got exchange message with unknown namespace '{}'",
                namespaced.namespace
            )
        }
    }

    async fn handle_module_requested_actions(
        &mut self,
        ws_messages: Vec<NamespacedEvent<'_, M::Outgoing>>,
        exchange_publish: Vec<ExchangePublish>,
        invalidate_data: bool,
        events: SelectAll<AnyStream>,
        exit: Option<CloseCode>,
    ) {
        for ws_message in ws_messages {
            self.interface
                .ws
                .send(WsMessageOutgoing::Module(ws_message.payload))
                .expect("Error sending outgoing module message");
        }

        for exchange_message in exchange_publish {
            self.exchange_sender
                .send(exchange_message)
                .expect("Error sending outgoing module message");
        }

        if invalidate_data {
            self.publish_exchange_control(control::exchange::Message::Update(self.participant_id))
                .expect("Error sending exchange participant-update message");
        }

        for event in events {
            self.events.push(event)
        }

        if let Some(exit) = exit {
            self.exit = true;

            log::debug!("Module requested exit with CloseCode: {:?}", exit);
        }
    }

    async fn leave_room(&mut self) -> Result<()> {
        let mut ws_messages = vec![];
        let mut exchange_publish = vec![];
        let mut invalidate_data = false;
        let mut events = SelectAll::new();
        let mut exit = None;

        let ctx = ModuleContext {
            role: self.role,
            timestamp: Timestamp::now(),
            ws_messages: &mut ws_messages,
            exchange_publish: &mut exchange_publish,
            redis_conn: &mut self.redis_conn,
            invalidate_data: &mut invalidate_data,
            events: &mut events,
            exit: &mut exit,
            metrics: None,
            m: PhantomData::<fn() -> M>,
        };

        self.module
            .on_event(ctx, Event::Leaving)
            .await
            .context("Module error on Leaving event")?;

        self.handle_module_requested_actions(
            ws_messages,
            exchange_publish,
            invalidate_data,
            events,
            exit,
        )
        .await;

        Ok(())
    }

    async fn build_participant(
        &mut self,
        id: ParticipantId,
    ) -> Result<types::signaling::control::Participant> {
        let mut participant = types::signaling::control::Participant {
            id,
            module_data: Default::default(),
        };

        let control_data = ControlState::from_redis(&mut self.redis_conn, self.room_id, id).await?;

        participant
            .module_data
            .insert(&control_data)
            .context("Failed to convert ControlData to serde_json::Value")?;

        Ok(participant)
    }

    async fn destroy(mut self) -> Result<()> {
        let mut set_lock = storage::room_mutex(self.room_id);

        let set_guard = set_lock.lock(&mut self.redis_conn).await?;

        storage::set_attribute(
            &mut self.redis_conn,
            self.room_id,
            self.participant_id,
            "left_at",
            Timestamp::now(),
        )
        .await?;

        let destroy_room =
            storage::participants_all_left(&mut self.redis_conn, self.room_id).await?;

        self.publish_exchange_control(control::exchange::Message::Left(self.participant_id))
            .context("Failed to send exchange participant-left message on destroy")?;

        let ctx = DestroyContext {
            redis_conn: &mut self.redis_conn.clone(),
            destroy_room,
        };
        let module = self.module;

        module.on_destroy(ctx).await;

        if destroy_room {
            storage::remove_attribute_key(&mut self.redis_conn, self.room_id, "display_name")
                .await?;
            storage::remove_attribute_key(&mut self.redis_conn, self.room_id, "kind").await?;
            storage::remove_attribute_key(&mut self.redis_conn, self.room_id, "joined_at").await?;

            storage::remove_attribute_key(&mut self.redis_conn, self.room_id, "hand_is_up").await?;
            storage::remove_attribute_key(&mut self.redis_conn, self.room_id, "hand_updated_at")
                .await?;

            storage::remove_attribute_key(&mut self.redis_conn, self.room_id, "user_id").await?;

            storage::remove_attribute_key(&mut self.redis_conn, self.room_id, "is_room_owner")
                .await?;
        }

        set_guard
            .unlock(&mut self.redis_conn)
            .await
            .context("Failed to unlock set_guard r3dlock while destroying mockrunner")
    }
}

/// Represents a WebSocket message sent from the Client to the Module
enum WsMessageIncoming<M>
where
    M: SignalingModule,
{
    Module(M::Incoming),
    Control(ControlCommand),
    /// The 'WebSocket' was closed
    CloseWs,
}

/// Represents a WebSocket message sent from the Module to the Client
#[allow(clippy::large_enum_variant)]
pub enum WsMessageOutgoing<M>
where
    M: SignalingModule,
{
    Module(M::Outgoing),
    Control(ControlEvent),
}

impl<M> Clone for WsMessageOutgoing<M>
where
    M: SignalingModule,
    M::Outgoing: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Module(outgoing) => Self::Module(outgoing.clone()),
            Self::Control(outgoing) => Self::Control(outgoing.clone()),
        }
    }
}

impl<M> std::fmt::Debug for WsMessageOutgoing<M>
where
    M: SignalingModule,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Module(arg0) => f.debug_tuple("Module").field(arg0).finish(),
            Self::Control(arg0) => f.debug_tuple("Control").field(arg0).finish(),
        }
    }
}

impl<M> PartialEq for WsMessageOutgoing<M>
where
    M: SignalingModule,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Module(l0), Self::Module(r0)) => l0 == r0,
            (Self::Control(l0), Self::Control(r0)) => l0 == r0,
            _ => false,
        }
    }
}

/// A interface used by the runner to interact with the client([`ModuleTester`])
struct ClientInterface<M>
where
    M: SignalingModule,
{
    ws: Interface<WsMessageOutgoing<M>, WsMessageIncoming<M>>,
}

/// A interface used by the client to interact with the runner([`MockRunner`])
struct RunnerInterface<M>
where
    M: SignalingModule,
{
    ws: Interface<WsMessageIncoming<M>, WsMessageOutgoing<M>>,
}

struct Interface<S, R> {
    sender: UnboundedSender<S>,
    receiver: UnboundedReceiver<R>,
}

impl<S, R> Interface<S, R> {
    fn new(sender: UnboundedSender<S>, receiver: UnboundedReceiver<R>) -> Self {
        Self { sender, receiver }
    }

    fn send(&self, value: S) -> Result<()> {
        self.sender
            .send(value)
            .map_err(|e| anyhow::Error::msg(format!("MockWs failed to send message, {e}")))
    }

    async fn recv(&mut self) -> Option<R> {
        self.receiver.recv().await
    }
}

/// Creates two interfaces that complement each other for bidirectional communication
///
/// eg.:
/// ``` text
/// Interface1 sending A and receiving B
/// Interface2 sending B and receiving A
/// ```
fn create_interface<A, B>() -> (Interface<A, B>, Interface<B, A>) {
    let (sender_a, receiver_a) = mpsc::unbounded_channel();
    let (sender_b, receiver_b) = mpsc::unbounded_channel();

    (
        Interface::new(sender_a, receiver_b),
        Interface::new(sender_b, receiver_a),
    )
}

/// Create the interfaces for the Client and Runner
async fn create_interfaces<M>() -> (ClientInterface<M>, RunnerInterface<M>)
where
    M: SignalingModule,
{
    let (ws_client_interface, ws_runner_interface) =
        create_interface::<WsMessageOutgoing<M>, WsMessageIncoming<M>>();

    let client_interface = ClientInterface {
        ws: ws_client_interface,
    };

    let runner_interface = RunnerInterface {
        ws: ws_runner_interface,
    };

    (client_interface, runner_interface)
}
