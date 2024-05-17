// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::BTreeSet,
    future,
    mem::replace,
    ops::ControlFlow,
    pin::Pin,
    sync::Arc,
    time::{Duration, Instant},
};

use actix::Addr;
use actix_http::ws::{CloseCode, CloseReason, Message};
use bytestring::ByteString;
use futures::{stream::SelectAll, Future};
use kustos::Authz;
use opentalk_controller_settings::SharedSettings;
use opentalk_database::{Db, DbConnection};
use opentalk_db_storage::{
    events::EventInvite, rooms::Room, tariffs::Tariff, users::User, utils::build_event_info,
};
use opentalk_signaling_core::{
    control::{
        self, exchange,
        storage::{self, AttributeActions as _, ControlStorage, ParticipantIdRunnerLock},
        ControlStateExt as _, NAMESPACE,
    },
    AnyStream, ExchangeHandle, ObjectStorage, Participant, RedisConnection, SignalingMetrics,
    SignalingModule, SignalingModuleError, SignalingRoomId, SubscriberHandle,
};
use opentalk_types::{
    common::tariff::TariffResource,
    core::{BreakoutRoomId, ParticipantId, ParticipationKind, UserId},
    signaling::{
        common::TargetParticipant,
        control::{
            command::ControlCommand,
            event::{self as control_event, ControlEvent, JoinBlockedReason, JoinSuccess},
            state::ControlState,
            AssociatedParticipant, Reason,
        },
        moderation::event::ModerationEvent,
        ModuleData, Role,
    },
};
use serde_json::Value;
use snafu::{ensure, whatever, Report, ResultExt, Snafu};
use tokio::{
    sync::{broadcast, mpsc},
    time::{interval, sleep},
};
use tokio_stream::StreamExt;
use uuid::Uuid;

use super::{
    actor::WebSocketActor,
    modules::{DynBroadcastEvent, DynEventCtx, DynTargetedEvent, Modules, NoSuchModuleError},
    DestroyContext, ExchangeBinding, ExchangePublish, NamespacedCommand, NamespacedEvent,
    RunnerMessage, Timestamp,
};
use crate::api::signaling::{
    echo::Echo,
    moderation,
    resumption::{ResumptionError, ResumptionTokenKeepAlive},
    trim_display_name,
    ws::actor::WsCommand,
};

mod call_in;

#[derive(Debug, Snafu)]
pub enum RunnerError {
    #[snafu(context(false), display("Couldn't get database connection."))]
    DbConnection {
        source: opentalk_database::DatabaseError,
    },

    #[snafu(context(false))]
    R3dLock {
        source: opentalk_r3dlock::Error,
    },

    #[snafu(context(false))]
    Redis {
        source: redis::RedisError,
    },

    #[snafu(context(false))]
    Signaling {
        source: SignalingModuleError,
    },

    InvalidDisplayName,

    #[snafu(display("InvalidState: {message}"))]
    InvalidState {
        message: String,
    },

    #[snafu(display("InvalidParticipant: {message}"))]
    InvalidParticipant {
        message: String,
    },

    #[snafu(whatever, display("Error: {message}"))]
    Other {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error + Sync + Send>, Some)))]
        source: Option<Box<dyn std::error::Error + Sync + Send>>,
    },
}

type Result<T, E = RunnerError> = std::result::Result<T, E>;

/// Builder to the runner type.
///
/// Passed into [`ModuleBuilder::build`](super::modules::ModuleBuilder::build) function to create an [`InitContext`](super::InitContext).
pub struct Builder {
    runner_id: Uuid,
    pub(super) id: ParticipantId,
    resuming: bool,
    pub(super) room: Room,
    pub(super) room_tariff: TariffResource,
    pub(super) breakout_room: Option<BreakoutRoomId>,
    pub(super) participant: Participant<User>,
    pub(super) role: Role,
    pub(super) protocol: &'static str,
    pub(super) metrics: Arc<SignalingMetrics>,
    pub(super) modules: Modules,
    pub(super) exchange_bindings: Vec<ExchangeBinding>,
    pub(super) events: SelectAll<AnyStream>,
    pub(super) db: Arc<Db>,
    pub(super) storage: Arc<ObjectStorage>,
    pub(super) authz: Arc<Authz>,
    pub(super) redis_conn: RedisConnection,
    pub(super) exchange_handle: ExchangeHandle,
    resumption_keep_alive: ResumptionTokenKeepAlive,
}

impl Builder {
    /// Abort the building process and destroy all already built modules
    #[tracing::instrument(skip(self))]
    pub async fn abort(mut self) {
        let ctx = DestroyContext {
            redis_conn: &mut self.redis_conn,
            // We haven't joined yet
            destroy_room: false,
        };

        self.modules.destroy(ctx).await
    }

    async fn acquire_participant_id(&mut self) -> Result<()> {
        let key = ParticipantIdRunnerLock { id: self.id };
        let runner_id = self.runner_id.to_string();

        // Try for up to 10 secs to acquire the key
        for _ in 0..10 {
            let value: redis::Value = redis::cmd("SET")
                .arg(&key)
                .arg(&runner_id)
                .arg("NX")
                .query_async(&mut self.redis_conn)
                .await?;

            match value {
                redis::Value::Nil => sleep(Duration::from_secs(1)).await,
                redis::Value::Okay => return Ok(()),
                _ => whatever!(
                    "Got unexpected value while acquiring runner id, value={:?}",
                    value
                ),
            }
        }

        whatever!("Failed to acquire runner id");
    }

    /// Build to runner from the data inside the builder and provided websocket
    #[tracing::instrument(err, skip_all)]
    pub async fn build(
        mut self,
        to_ws_actor: Addr<WebSocketActor>,
        from_ws_actor: mpsc::UnboundedReceiver<RunnerMessage>,
        shutdown_sig: broadcast::Receiver<()>,
        settings: SharedSettings,
    ) -> Result<Runner> {
        self.acquire_participant_id().await?;

        let room_id = SignalingRoomId::new(self.room.id, self.breakout_room);

        // Create list of routing keys that address this runner
        let mut routing_keys = vec![
            exchange::current_room_all_participants(room_id),
            exchange::global_room_all_participants(room_id.room_id()),
            exchange::current_room_by_participant_id(room_id, self.id),
            exchange::global_room_by_participant_id(room_id.room_id(), self.id),
        ];

        match self.participant {
            Participant::User(ref user) => {
                routing_keys.push(exchange::current_room_by_user_id(room_id, user.id));
                routing_keys.push(exchange::global_room_by_user_id(room_id.room_id(), user.id));
            }
            Participant::Recorder => {
                routing_keys.push(exchange::current_room_all_recorders(room_id))
            }
            Participant::Guest | Participant::Sip => {}
        }

        for ExchangeBinding { routing_key } in self.exchange_bindings {
            routing_keys.push(routing_key);
        }

        let subscriber_handle =
            self.exchange_handle
                .create_subscriber(routing_keys)
                .await
                .whatever_context::<_, RunnerError>("Failed to create subscriber")?;

        self.resumption_keep_alive
            .set_initial(&mut self.redis_conn)
            .await?;

        Ok(Runner {
            runner_id: self.runner_id,
            id: self.id,
            resuming: self.resuming,
            room: self.room,
            room_id,
            participant: self.participant,
            role: self.role,
            state: RunnerState::None,
            ws: Ws {
                to_actor: to_ws_actor,
                from_actor: from_ws_actor,
                state: State::Open,
            },
            modules: self.modules,
            events: self.events,
            metrics: self.metrics,
            db: self.db,
            redis_conn: self.redis_conn,
            exchange_handle: self.exchange_handle,
            subscriber_handle,
            resumption_keep_alive: self.resumption_keep_alive,
            shutdown_sig,
            exit: false,
            settings,
            time_limit_future: Box::pin(future::pending()),
        })
    }
}

/// The session runner
///
/// As root of the session-task it is responsible to drive the module,
/// manage setup and teardown of redis storage, the exchange subscriber and modules.
///
/// Also acts as `control` module which handles participant and room states.
pub struct Runner {
    /// Runner ID which is used to assume ownership of a participant id
    runner_id: Uuid,

    /// participant id that the runner is connected to
    id: ParticipantId,

    /// True if a resumption token was used
    resuming: bool,

    /// The database repr of the current room at the time of joining
    room: Room,

    /// Full signaling room id
    room_id: SignalingRoomId,

    /// User behind the participant or Guest
    participant: Participant<User>,

    /// The role of the participant inside the room
    role: Role,

    /// The control data. Initialized when frontend send join
    state: RunnerState,

    /// Websocket abstraction which connects the to the websocket actor
    ws: Ws,

    /// All registered and initialized modules
    modules: Modules,
    events: SelectAll<AnyStream>,

    /// Signaling metrics for this runner
    metrics: Arc<SignalingMetrics>,

    /// Database connection pool
    db: Arc<Db>,

    /// Redis connection manager
    redis_conn: RedisConnection,

    /// Exchange handle - used to send messages
    exchange_handle: ExchangeHandle,

    /// Exchange Subscriber - channel we receive all messages from
    subscriber_handle: SubscriberHandle,

    /// Util to keep the resumption token alive
    resumption_keep_alive: ResumptionTokenKeepAlive,

    /// global application shutdown signal
    shutdown_sig: broadcast::Receiver<()>,

    /// When set to true the runner will gracefully exit on next loop
    exit: bool,

    /// Shared settings of the running program
    settings: SharedSettings,

    time_limit_future: Pin<Box<dyn Future<Output = ()>>>,
}

/// Current state of the runner
#[derive(Debug, Clone, PartialEq, Eq)]
enum RunnerState {
    /// Runner and its message exchange resources are created
    /// but has not joined the room yet (no redis resources set)
    None,

    /// Inside the waiting room
    Waiting {
        accepted: bool,
        control_data: ControlState,
    },

    /// Inside the actual room
    Joined,
}

async fn get_participant_role(
    conn: &mut DbConnection,
    participant: &Participant<User>,
    room: &Room,
) -> Result<Role> {
    let user = if let Participant::User(user) = participant {
        user
    } else {
        return Ok(Role::Guest);
    };

    if user.id == room.created_by {
        return Ok(Role::Moderator);
    }

    match EventInvite::get_for_user_and_room(conn, user.id, room.id)
        .await
        .whatever_context::<_, RunnerError>("Failed to get invite events")?
    {
        Some(event_invite) => Ok(event_invite.role.into()),
        None => Ok(Role::User),
    }
}

impl Runner {
    #[allow(clippy::too_many_arguments)]
    pub async fn builder(
        runner_id: Uuid,
        id: ParticipantId,
        resuming: bool,
        room: Room,
        room_tariff: TariffResource,
        breakout_room: Option<BreakoutRoomId>,
        participant: Participant<User>,
        protocol: &'static str,
        metrics: Arc<SignalingMetrics>,
        db: Arc<Db>,
        storage: Arc<ObjectStorage>,
        authz: Arc<Authz>,
        redis_conn: RedisConnection,
        exchange_handle: ExchangeHandle,
        resumption_keep_alive: ResumptionTokenKeepAlive,
    ) -> Result<Builder> {
        let role = get_participant_role(&mut db.get_conn().await?, &participant, &room).await?;

        Ok(Builder {
            runner_id,
            id,
            resuming,
            room,
            room_tariff,
            breakout_room,
            participant,
            role,
            protocol,
            metrics,
            modules: Default::default(),
            exchange_bindings: vec![],
            events: SelectAll::new(),
            db,
            storage,
            authz,
            redis_conn,
            exchange_handle,
            resumption_keep_alive,
        })
    }

    /// Destroys the runner and all associated resources
    #[tracing::instrument(skip(self), fields(id = %self.id))]
    pub async fn destroy(mut self, close_ws: bool, reason: Reason) {
        let destroy_start_time = Instant::now();
        let mut encountered_error = false;

        if let RunnerState::Joined | RunnerState::Waiting { .. } = &self.state {
            // The retry/wait_time values are set extra high
            // since a lot of operations are being done while holding the lock
            let mut room_mutex = storage::room_mutex(self.room_id);

            let room_guard = match room_mutex.lock(&mut self.redis_conn).await {
                Ok(guard) => guard,
                Err(opentalk_r3dlock::Error::Redis { source: e }) => {
                    log::error!("Failed to acquire r3dlock, {}", Report::from_error(e));
                    // There is a problem when accessing redis which could
                    // mean either the network or redis is broken.
                    // Both cases cannot be handled here, abort the cleanup

                    self.metrics
                        .record_destroy_time(destroy_start_time.elapsed().as_secs_f64(), false);

                    return;
                }
                Err(opentalk_r3dlock::Error::CouldNotAcquireLock) => {
                    log::error!("Failed to acquire r3dlock, contention too high");

                    self.metrics
                        .record_destroy_time(destroy_start_time.elapsed().as_secs_f64(), false);

                    return;
                }
                Err(
                    opentalk_r3dlock::Error::FailedToUnlock
                    | opentalk_r3dlock::Error::AlreadyExpired,
                ) => {
                    unreachable!()
                }
            };

            if let RunnerState::Joined = &self.state {
                let res = self
                    .redis_conn
                    .set_attribute(self.room_id, self.id, "left_at", Timestamp::now())
                    .await;
                if let Err(e) = res {
                    log::error!(
                        "failed to mark participant as left, {}",
                        Report::from_error(e)
                    );
                    encountered_error = true;
                }
            } else if let RunnerState::Waiting { .. } = &self.state {
                if let Err(e) = moderation::storage::waiting_room_remove(
                    &mut self.redis_conn,
                    self.room_id.room_id(),
                    self.id,
                )
                .await
                {
                    log::error!(
                        "failed to remove participant from waiting_room list, {:?}",
                        e
                    );
                    encountered_error = true;
                }
                if let Err(e) = moderation::storage::waiting_room_accepted_remove(
                    &mut self.redis_conn,
                    self.room_id.room_id(),
                    self.id,
                )
                .await
                {
                    log::error!(
                        "failed to remove participant from waiting_room_accepted list, {:?}",
                        e
                    );
                    encountered_error = true;
                }
            };

            let room_is_empty = match self.redis_conn.participants_all_left(self.room_id).await {
                Ok(room_is_empty) => room_is_empty,
                Err(e) => {
                    log::error!("Failed to check if room is empty {}", Report::from_error(e));
                    encountered_error = true;
                    false
                }
            };

            // if the room is empty check that the waiting room is empty
            let destroy_room = if room_is_empty {
                if self.room_id.breakout_room_id().is_some() {
                    // Breakout rooms are destroyed even with participants inside the waiting room
                    true
                } else {
                    // destroy room only if waiting room is empty
                    let waiting_room_is_empty = match moderation::storage::waiting_room_len(
                        &mut self.redis_conn,
                        self.room_id.room_id(),
                    )
                    .await
                    {
                        Ok(waiting_room_len) => waiting_room_len == 0,
                        Err(e) => {
                            log::error!(
                                "failed to get waiting room len, {}",
                                Report::from_error(e)
                            );
                            encountered_error = true;
                            false
                        }
                    };
                    let waiting_room_accepted_is_empty =
                        match moderation::storage::waiting_room_accepted_len(
                            &mut self.redis_conn,
                            self.room_id.room_id(),
                        )
                        .await
                        {
                            Ok(waiting_room_len) => waiting_room_len == 0,
                            Err(e) => {
                                log::error!(
                                    "failed to get accepted waiting room len, {}",
                                    Report::from_error(e)
                                );
                                encountered_error = true;
                                false
                            }
                        };
                    waiting_room_is_empty && waiting_room_accepted_is_empty
                }
            } else {
                false
            };

            match self
                .redis_conn
                .decrement_participant_count(self.room.id)
                .await
            {
                Ok(remaining_participant_count) => {
                    if remaining_participant_count == 0 {
                        if let Err(e) = self.cleanup_redis_for_global_room().await {
                            log::error!(
                                "failed to mark participant as left, {}",
                                Report::from_error(e)
                            );
                            encountered_error = true;
                        }
                    }
                }
                Err(e) => {
                    log::error!(
                        "failed to decrement participant count, {}",
                        Report::from_error(e)
                    );
                    encountered_error = true;
                }
            }

            let ctx = DestroyContext {
                redis_conn: &mut self.redis_conn,
                destroy_room,
            };

            self.modules.destroy(ctx).await;

            if destroy_room {
                if let Err(e) = self.cleanup_redis_keys_for_current_room().await {
                    log::error!(
                        "Failed to remove all control attributes, {}",
                        Report::from_error(e)
                    );
                    encountered_error = true;
                }

                self.metrics.increment_destroyed_rooms_count();
            }

            self.metrics.decrement_participants_count(&self.participant);

            if let Err(e) = room_guard.unlock(&mut self.redis_conn).await {
                log::error!(
                    "Failed to unlock set_guard r3dlock, {}",
                    Report::from_error(e)
                );
                encountered_error = true;
            }

            if !destroy_room {
                match &self.state {
                    RunnerState::None => unreachable!("state was checked before"),
                    RunnerState::Waiting { .. } => {
                        self.exchange_publish(
                            exchange::global_room_all_participants(self.room_id.room_id()),
                            serde_json::to_string(&NamespacedEvent {
                                namespace: moderation::NAMESPACE,
                                timestamp: Timestamp::now(),
                                payload: moderation::exchange::Message::LeftWaitingRoom(self.id),
                            })
                            .expect("Failed to convert namespaced to json"),
                        );
                    }
                    RunnerState::Joined => {
                        // Skip sending the left message.
                        // TODO:(kbalt): The left message is the only message not sent by the recorder, all other
                        // messages are currently ignored by filtering in the `build_participant` function
                        // It'd might be nicer to have a "visibility" check before sending any "joined"/"updated"/"left"
                        // message
                        if !matches!(&self.participant, Participant::Recorder) {
                            self.exchange_publish_control(
                                Timestamp::now(),
                                None,
                                exchange::Message::Left {
                                    id: self.id,
                                    reason,
                                },
                            );
                        }
                    }
                }
            }
        } else {
            // Not joined, just destroy modules normal
            let ctx = DestroyContext {
                redis_conn: &mut self.redis_conn,
                destroy_room: false,
            };

            self.modules.destroy(ctx).await;
        }

        // release participant id
        match redis::cmd("GETDEL")
            .arg(ParticipantIdRunnerLock { id: self.id })
            .query_async::<_, String>(&mut self.redis_conn)
            .await
        {
            Ok(runner_id) => {
                if runner_id != self.runner_id.to_string() {
                    log::warn!("removed runner id does not match the id of the runner");
                }
            }
            Err(e) => {
                log::error!("failed to remove participant id, {}", Report::from_error(e));
                encountered_error = true;
            }
        }

        self.metrics.record_destroy_time(
            destroy_start_time.elapsed().as_secs_f64(),
            !encountered_error,
        );

        // If a Close frame is received from the websocket actor, manually return a close command
        if close_ws {
            self.ws.close(CloseCode::Normal).await;
        }
    }

    /// Remove all room and control module related data from redis for the current 'local' room/breakout-room. Does not
    /// touch any keys that contain 'global' data that is used across all 'sub'-rooms (main & breakout rooms).
    async fn cleanup_redis_keys_for_current_room(&mut self) -> Result<()> {
        self.redis_conn.remove_room_closes_at(self.room_id).await?;
        self.redis_conn.remove_participant_set(self.room_id).await?;
        for key in [
            "display_name",
            "role",
            "joined_at",
            "left_at",
            "hand_is_up",
            "hand_updated_at",
            "kind",
            "user_id",
            "avatar_url",
        ] {
            self.redis_conn
                .remove_attribute_key(self.room_id, key)
                .await?;
        }

        Ok(())
    }

    /// Remove all room and control module related redis keys that are used across all 'sub'-rooms. This must only be
    /// called once the main and all breakout rooms are empty.
    async fn cleanup_redis_for_global_room(&mut self) -> Result<()> {
        self.redis_conn
            .delete_participant_count(self.room.id)
            .await?;
        self.redis_conn.delete_tariff(self.room.id).await?;
        self.redis_conn.delete_event(self.room.id).await?;

        Ok(())
    }

    /// Runs the runner until the peer closes its websocket connection or a fatal error occurs.
    pub async fn run(mut self) {
        let mut manual_close_ws = false;

        // Set default `skip_waiting_room` key value with the default expiration time
        _ = storage::set_skip_waiting_room_with_expiry_nx(&mut self.redis_conn, self.id, false)
            .await;
        let mut skip_waiting_room_refresh_interval = interval(Duration::from_secs(
            opentalk_signaling_core::control::storage::SKIP_WAITING_ROOM_KEY_REFRESH_INTERVAL,
        ));

        let mut reason = Reason::Quit;

        while matches!(self.ws.state, State::Open) {
            if self.exit && matches!(self.ws.state, State::Open) {
                // This case handles exit on errors unrelated to websocket or controller shutdown
                self.ws.close(CloseCode::Abnormal).await;
            }

            tokio::select! {
                res = self.ws.receive() => {
                    match res {
                        Some(RunnerMessage::Timeout) => reason = Reason::Timeout,
                        Some(RunnerMessage::Message(Message::Close(_))) => {
                            // Received Close frame from ws actor, break to destroy the runner
                            manual_close_ws = true;
                            break;
                        }
                        Some(RunnerMessage::Message(msg)) => {
                            self.handle_ws_message(msg).await;
                        }
                        None => {
                            // Ws is now going to be in error state and cause the runner to exit
                            log::error!("Failed to receive ws message for participant {}", self.id);
                        }
                    }
                }
                msg = self.subscriber_handle.receive() => {
                    match msg {
                        Some(msg) => self.handle_exchange_msg(msg).await,
                        None => {
                            // The message exchange dropped our subscriber, error out and exit
                            self.exit = true
                        },
                    }
                }
                Some((namespace, any)) = self.events.next() => {
                    let timestamp = Timestamp::now();
                    let actions = self.handle_module_targeted_event(namespace, timestamp, DynTargetedEvent::Ext(any))
                        .await
                        .expect("Should not get events from unknown modules");

                    self.handle_module_requested_actions(timestamp, actions).await;
                }
                _ = skip_waiting_room_refresh_interval.tick() => {
                    _ = storage::reset_skip_waiting_room_expiry(
                        &mut self.redis_conn,
                        self.id,
                    )
                    .await;
                }
                _ = &mut self.time_limit_future => {
                    self.ws_send_control(Timestamp::now(), ControlEvent::TimeLimitQuotaElapsed).await;
                    self.ws.close(CloseCode::Normal).await;
                    break;
                }
                _ = self.shutdown_sig.recv() => {
                    self.ws.close(CloseCode::Away).await;
                    break;
                }
                _ = self.resumption_keep_alive.wait() => {
                    match self.resumption_keep_alive.refresh(&mut self.redis_conn).await {
                        Ok(_) => {},
                        Err(ResumptionError::Used) => {
                            log::warn!("Closing connection of this runner as its resumption token was used");

                            self.ws.close(CloseCode::Normal).await;
                        },
                        Err(e) => {
                            log::error!("failed to set resumption token in redis, {}", Report::from_error(e));
                        }
                    }
                }
            }
        }
        let timestamp = Timestamp::now();
        let actions = self
            .handle_module_broadcast_event(timestamp, DynBroadcastEvent::Leaving, false)
            .await;

        self.handle_module_requested_actions(timestamp, actions)
            .await;

        log::debug!("Stopping ws-runner task for participant {}", self.id);

        self.destroy(manual_close_ws, reason).await;
    }

    #[tracing::instrument(skip(self, message), fields(id = %self.id))]
    async fn handle_ws_message(&mut self, message: Message) {
        log::trace!("Received websocket message {:?}", message);

        let value: Result<NamespacedCommand<'_, Value>, _> = match message {
            Message::Text(ref text) => serde_json::from_str(text),
            Message::Binary(ref binary) => serde_json::from_slice(binary),
            _ => unreachable!(),
        };

        let timestamp = Timestamp::now();

        let namespaced = match value {
            Ok(value) => value,
            Err(e) => {
                log::error!(
                    "Failed to parse namespaced message, {}",
                    Report::from_error(e)
                );

                self.ws_send_control_error(timestamp, control_event::Error::InvalidJson)
                    .await;

                return;
            }
        };

        if namespaced.namespace == NAMESPACE {
            match serde_json::from_value(namespaced.payload) {
                Ok(msg) => {
                    if let Err(e) = self.handle_control_msg(timestamp, msg).await {
                        log::error!("Failed to handle control msg, {}", Report::from_error(e));
                        self.exit = true;
                    }
                }
                Err(e) => {
                    log::error!("Failed to parse control payload, {}", Report::from_error(e));

                    self.ws_send_control_error(timestamp, control_event::Error::InvalidJson)
                        .await;
                }
            }
            // Do not handle any other messages than control-join or echo before joined
        } else if matches!(&self.state, RunnerState::Joined)
            || matches!(namespaced.namespace, Echo::NAMESPACE)
        {
            match self
                .handle_module_targeted_event(
                    namespaced.namespace,
                    timestamp,
                    DynTargetedEvent::WsMessage(namespaced.payload),
                )
                .await
            {
                Ok(actions) => {
                    self.handle_module_requested_actions(timestamp, actions)
                        .await
                }
                Err(NoSuchModuleError) => {
                    self.ws_send_control_error(timestamp, control_event::Error::InvalidNamespace)
                        .await;
                }
            }
        }
    }

    async fn handle_control_msg(
        &mut self,
        timestamp: Timestamp,
        msg: ControlCommand,
    ) -> Result<()> {
        match msg {
            ControlCommand::Join(join) => {
                if !matches!(self.state, RunnerState::None) {
                    self.ws_send_control_error(timestamp, control_event::Error::AlreadyJoined)
                        .await;

                    return Ok(());
                }

                let control_data = match self.query_control_data(join.display_name, timestamp).await
                {
                    Err(RunnerError::InvalidDisplayName) => {
                        self.ws_send_control_error(
                            timestamp,
                            control_event::Error::InvalidUsername,
                        )
                        .await;

                        return Ok(());
                    }
                    other => other,
                }?;

                self.metrics.increment_participants_count(&self.participant);

                // Allow moderators, invisible services, and already accepted participants to skip the waiting room
                let can_skip_waiting_room: bool =
                    storage::get_skip_waiting_room(&mut self.redis_conn, self.id).await?;

                let skip_waiting_room = matches!(self.role, Role::Moderator)
                    || !control_data.participation_kind.is_visible()
                    || can_skip_waiting_room;

                let waiting_room_enabled = moderation::storage::init_waiting_room_key(
                    &mut self.redis_conn,
                    self.room_id.room_id(),
                    self.room.waiting_room,
                )
                .await?;

                if !skip_waiting_room && waiting_room_enabled {
                    // Waiting room is enabled; join the waiting room
                    self.join_waiting_room(timestamp, control_data).await?;
                } else {
                    // Waiting room is not enabled; join the room directly
                    self.join_room(timestamp, control_data, false).await?;
                }
            }
            ControlCommand::EnterRoom => {
                match replace(&mut self.state, RunnerState::None) {
                    RunnerState::Waiting {
                        accepted: true,
                        control_data,
                    } => {
                        self.exchange_publish(
                            control::exchange::global_room_all_participants(self.room_id.room_id()),
                            serde_json::to_string(&NamespacedEvent {
                                namespace: moderation::NAMESPACE,
                                timestamp,
                                payload: moderation::exchange::Message::LeftWaitingRoom(self.id),
                            })
                            .expect("Failed to convert namespaced to json"),
                        );

                        moderation::storage::waiting_room_accepted_remove(
                            &mut self.redis_conn,
                            self.room_id.room_id(),
                            self.id,
                        )
                        .await?;

                        self.join_room(timestamp, control_data, true).await?
                    }
                    // not in correct state, reset it
                    state => {
                        self.state = state;

                        self.ws_send_control_error(
                            timestamp,
                            control_event::Error::NotAcceptedOrNotInWaitingRoom,
                        )
                        .await;
                    }
                }
            }
            ControlCommand::RaiseHand => {
                if !moderation::storage::is_raise_hands_enabled(&mut self.redis_conn, self.room.id)
                    .await?
                {
                    self.ws_send_control_error(timestamp, control_event::Error::RaiseHandsDisabled)
                        .await;

                    return Ok(());
                }

                self.handle_raise_hand_change(timestamp, true).await?;
                self.ws_send_control(timestamp, ControlEvent::HandRaised)
                    .await;
            }
            ControlCommand::LowerHand => {
                self.handle_raise_hand_change(timestamp, false).await?;
                self.ws_send_control(timestamp, ControlEvent::HandLowered)
                    .await;
            }
            ControlCommand::GrantModeratorRole(TargetParticipant { target }) => {
                if !matches!(self.state, RunnerState::Joined) {
                    self.ws_send_control_error(timestamp, control_event::Error::NotYetJoined)
                        .await;

                    return Ok(());
                }

                self.handle_grant_moderator_msg(timestamp, target, true)
                    .await?;
            }
            ControlCommand::RevokeModeratorRole(TargetParticipant { target }) => {
                if !matches!(self.state, RunnerState::Joined) {
                    self.ws_send_control_error(timestamp, control_event::Error::NotYetJoined)
                        .await;

                    return Ok(());
                }

                self.handle_grant_moderator_msg(timestamp, target, false)
                    .await?;
            }
        }

        Ok(())
    }

    async fn query_control_data(
        &mut self,
        join_display_name: Option<String>,
        timestamp: Timestamp,
    ) -> Result<ControlState, RunnerError> {
        let display_name = self.username_or(join_display_name).await;
        let avatar_url = self.avatar_url().await;

        if display_name.is_empty() || display_name.len() > 100 {
            return InvalidDisplayNameSnafu.fail();
        }

        let left_at: Option<Timestamp> = self
            .redis_conn
            .get_attribute(self.room_id, self.id, "left_at")
            .await?;
        self.set_control_attributes(timestamp, &display_name, avatar_url.as_deref())
            .await?;

        Ok(ControlState {
            display_name,
            role: self.role,
            avatar_url,
            participation_kind: self.participant.kind(),
            joined_at: timestamp,
            hand_is_up: false,
            hand_updated_at: timestamp,
            left_at,
            is_room_owner: self.participant.user_id() == Some(self.room.created_by),
        })
    }

    async fn handle_grant_moderator_msg(
        &mut self,
        timestamp: Timestamp,
        target: ParticipantId,
        grant: bool,
    ) -> Result<()> {
        if self.role != Role::Moderator {
            self.ws_send_control_error(timestamp, control_event::Error::InsufficientPermissions)
                .await;

            return Ok(());
        }

        let role: Option<Role> = self
            .redis_conn
            .get_attribute(self.room_id, target, "role")
            .await?;

        let is_moderator = matches!(role, Some(Role::Moderator));

        if is_moderator == grant {
            self.ws_send_control_error(timestamp, control_event::Error::NothingToDo)
                .await;

            return Ok(());
        }

        let user_id: Option<UserId> = self
            .redis_conn
            .get_attribute(self.room_id, target, "user_id")
            .await?;

        if let Some(user_id) = user_id {
            if user_id == self.room.created_by {
                self.ws_send_control_error(timestamp, control_event::Error::TargetIsRoomOwner)
                    .await;

                return Ok(());
            }
        }

        self.exchange_publish_control(
            timestamp,
            Some(target),
            exchange::Message::SetModeratorStatus(grant),
        );

        Ok(())
    }

    async fn handle_raise_hand_change(
        &mut self,
        timestamp: Timestamp,
        hand_raised: bool,
    ) -> Result<()> {
        self.redis_conn
            .bulk_attribute_actions(self.room_id, self.id)
            .set("hand_is_up", hand_raised)
            .set("hand_updated_at", timestamp)
            .apply(&mut self.redis_conn)
            .await?;

        let broadcast_event = if hand_raised {
            DynBroadcastEvent::RaiseHand
        } else {
            DynBroadcastEvent::LowerHand
        };
        let actions = self
            .handle_module_broadcast_event(timestamp, broadcast_event, true)
            .await;

        self.handle_module_requested_actions(timestamp, actions)
            .await;

        Ok(())
    }

    async fn room_has_moderator_besides_me(&mut self) -> Result<bool> {
        let roles_and_left_at_timestamps = self
            .redis_conn
            .get_role_and_left_at_for_room_participants(SignalingRoomId::new_for_room(self.room.id))
            .await?;

        Ok(roles_and_left_at_timestamps
            .iter()
            .filter(|(k, _)| **k != self.id)
            .any(|(_, (role, left_at))| {
                role.as_ref().map(Role::is_moderator).unwrap_or_default() && left_at.is_none()
            }))
    }

    /// Enforces the given tariff.
    ///
    /// Requires the room lock to be taken before calling
    async fn enforce_tariff(
        &mut self,
        tariff: Tariff,
    ) -> Result<ControlFlow<JoinBlockedReason, Tariff>> {
        let tariff = self
            .redis_conn
            .try_init_tariff(self.room.id, tariff)
            .await?;

        if self.role == Role::Moderator && !self.room_has_moderator_besides_me().await? {
            self.redis_conn
                .increment_participant_count(self.room.id)
                .await?;
            return Ok(ControlFlow::Continue(tariff));
        }

        if let Some(participant_limit) = tariff.quotas.0.get("room_participant_limit") {
            if let Some(count) = self.redis_conn.get_participant_count(self.room.id).await? {
                if count >= *participant_limit as isize {
                    return Ok(ControlFlow::Break(
                        JoinBlockedReason::ParticipantLimitReached,
                    ));
                }
            }
        }

        self.redis_conn
            .increment_participant_count(self.room.id)
            .await?;

        Ok(ControlFlow::Continue(tariff))
    }

    async fn join_waiting_room(
        &mut self,
        timestamp: Timestamp,
        control_data: ControlState,
    ) -> Result<()> {
        let db = self.db.clone();
        let creator_id = self.room.created_by;

        let tariff = Tariff::get_by_user_id(&mut db.get_conn().await?, &creator_id)
            .await
            .whatever_context::<_, RunnerError>("Failed to get user")?;

        let mut lock = storage::room_mutex(self.room_id);
        let guard = lock.lock(&mut self.redis_conn).await?;

        match self.enforce_tariff(tariff).await {
            Ok(ControlFlow::Continue(_)) => { /* continue */ }
            Ok(ControlFlow::Break(reason)) => {
                guard.unlock(&mut self.redis_conn).await?;

                self.ws_send_control(Timestamp::now(), ControlEvent::JoinBlocked(reason))
                    .await;

                return Ok(());
            }
            Err(e) => {
                guard.unlock(&mut self.redis_conn).await?;

                return Err(e);
            }
        };

        let res = moderation::storage::waiting_room_add(
            &mut self.redis_conn,
            self.room_id.room_id(),
            self.id,
        )
        .await;

        guard.unlock(&mut self.redis_conn).await?;
        let num_added = res?;

        // Check that SADD doesn't return 0. That would mean that the participant id would be a
        // duplicate which cannot be allowed. Since this should never happen just error and exit.
        if !self.resuming && num_added == 0 {
            whatever!("participant-id is already taken inside waiting-room set");
        }

        self.state = RunnerState::Waiting {
            accepted: false,
            control_data,
        };

        self.ws
            .send(Message::Text(
                serde_json::to_string(&NamespacedEvent {
                    namespace: moderation::NAMESPACE,
                    timestamp,
                    payload: ModerationEvent::InWaitingRoom,
                })
                .whatever_context::<_, RunnerError>("Failed to send")?
                .into(),
            ))
            .await;

        self.exchange_publish(
            control::exchange::global_room_all_participants(self.room_id.room_id()),
            serde_json::to_string(&NamespacedEvent {
                namespace: moderation::NAMESPACE,
                timestamp,
                payload: moderation::exchange::Message::JoinedWaitingRoom(self.id),
            })
            .expect("Failed to convert namespaced to json"),
        );

        Ok(())
    }

    async fn join_room(
        &mut self,
        timestamp: Timestamp,
        control_data: ControlState,
        joining_from_waiting_room: bool,
    ) -> Result<()> {
        let mut lock = storage::room_mutex(self.room_id);

        // Clear the left_at timestamp to indicate that the participant is in the real room (not just the waiting room).
        let control_data = ControlState {
            left_at: None,
            ..control_data
        };
        self.redis_conn
            .remove_attribute(self.room_id, self.id, "left_at")
            .await?;

        // If we haven't joined the waiting room yet, fetch, set and enforce the tariff for the room.
        // When in waiting-room this logic was already executed in `join_waiting_room`.
        let (guard, tariff) = if !joining_from_waiting_room {
            let creator_id = self.room.created_by;

            let mut tariff = Tariff::get_by_user_id(&mut self.db.get_conn().await?, &creator_id)
                .await
                .whatever_context::<_, RunnerError>("Failed to get user")?;

            let guard = lock.lock(&mut self.redis_conn).await?;

            match self.enforce_tariff(tariff.clone()).await {
                Ok(ControlFlow::Continue(enforced_tariff)) => {
                    tariff = enforced_tariff;
                }
                Ok(ControlFlow::Break(reason)) => {
                    guard.unlock(&mut self.redis_conn).await?;

                    self.ws_send_control(Timestamp::now(), ControlEvent::JoinBlocked(reason))
                        .await;

                    return Ok(());
                }
                Err(e) => {
                    guard.unlock(&mut self.redis_conn).await?;

                    return Err(e);
                }
            };

            (guard, tariff)
        } else {
            let tariff = self.redis_conn.get_tariff(self.room.id).await?;
            (lock.lock(&mut self.redis_conn).await?, tariff)
        };

        let res = self.join_room_locked().await;

        let unlock_res = guard.unlock(&mut self.redis_conn).await;

        let participant_ids = match res {
            Ok(participants) => participants,
            Err(e) => {
                whatever!("Failed to join room, {e:?}\nUnlocked room lock, {unlock_res:?}");
            }
        };

        unlock_res?;

        let event = opentalk_db_storage::events::Event::get_for_room(
            &mut self.db.get_conn().await?,
            self.room.id,
        )
        .await
        .whatever_context::<_, RunnerError>("Failed to get first event for room")?;
        let event = self.redis_conn.try_init_event(self.room.id, event).await?;

        let mut participants = vec![];

        for id in participant_ids {
            if self.id == id {
                continue;
            }

            match self.build_participant(id).await {
                Ok(Some(participant)) => participants.push(participant),
                Ok(None) => { /* ignore invisible participants */ }
                Err(e) => log::error!(
                    "Failed to build participant {}, {}",
                    id,
                    Report::from_error(e)
                ),
            };
        }

        let mut module_data = ModuleData::new();

        let actions = self
            .handle_module_broadcast_event(
                timestamp,
                DynBroadcastEvent::Joined(&control_data, &mut module_data, &mut participants),
                false,
            )
            .await;

        let closes_at = self.redis_conn.get_room_closes_at(self.room_id).await?;

        let settings = self.settings.load_full();

        let mut module_features = Vec::<(&str, Vec<&str>)>::new();
        self.modules
            .get_module_features()
            .iter()
            .for_each(|(k, v)| {
                module_features.push((k, v.clone()));
            });

        let tariff_resource = tariff
            .to_tariff_resource(settings.defaults.disabled_features(), module_features)
            .into();

        let mut conn = self.db.get_conn().await?;
        let room_id = self.room.id;
        let event_info = match event.as_ref() {
            Some(event) => {
                let call_in_tel = settings.call_in.as_ref().map(|call_in| call_in.tel.clone());
                Some(build_event_info(&mut conn, call_in_tel, room_id, event).await?)
            }
            _ => None,
        };

        self.ws_send_control(
            timestamp,
            ControlEvent::JoinSuccess(JoinSuccess {
                id: self.id,
                display_name: control_data.display_name.clone(),
                avatar_url: control_data.avatar_url.clone(),
                role: self.role,
                closes_at,
                tariff: tariff_resource,
                module_data,
                participants,
                event_info,
                is_room_owner: self.participant.user_id() == Some(self.room.created_by),
            }),
        )
        .await;

        self.state = RunnerState::Joined;

        self.exchange_publish_control(timestamp, None, exchange::Message::Joined(self.id));

        self.handle_module_requested_actions(timestamp, actions)
            .await;

        Ok(())
    }

    async fn join_room_locked(&mut self) -> Result<BTreeSet<ParticipantId>> {
        let participant_set_exists = self.redis_conn.participant_set_exists(self.room_id).await?;

        if !participant_set_exists {
            self.set_room_time_limit().await?;
            self.metrics.increment_created_rooms_count();
        }
        self.activate_room_time_limit().await?;

        let participants = self.redis_conn.get_all_participants(self.room_id).await?;

        let added = self
            .redis_conn
            .add_participant_to_set(self.room_id, self.id)
            .await?;

        // Check that SADD doesn't return 0. That would mean that the participant id would be a
        // duplicate which cannot be allowed. Since this should never happen just error and exit.
        if !self.resuming && !added {
            whatever!("participant-id is already taken inside participant set");
        }

        Ok(participants)
    }

    async fn set_room_time_limit(&mut self) -> Result<()> {
        let tariff = self.redis_conn.get_tariff(self.room.id).await?;

        let quotas = tariff.quotas.0;
        let remaining_seconds = quotas
            .get("room_time_limit_secs")
            .map(|time_limit| *time_limit as i64);

        if let Some(remaining_seconds) = remaining_seconds {
            let closes_at =
                Timestamp::now().checked_add_signed(chrono::Duration::seconds(remaining_seconds));

            if let Some(closes_at) = closes_at {
                self.redis_conn
                    .set_room_closes_at(self.room_id, closes_at.into())
                    .await?;
            } else {
                log::error!("DateTime overflow for closes_at");
            }
        }

        Ok(())
    }

    async fn activate_room_time_limit(&mut self) -> Result<()> {
        let closes_at = self.redis_conn.get_room_closes_at(self.room_id).await?;

        if let Some(closes_at) = closes_at {
            let remaining_seconds = (*closes_at - *Timestamp::now()).num_seconds();
            let future = tokio::time::sleep(Duration::from_secs(remaining_seconds.max(0) as u64));
            self.time_limit_future = Box::pin(future);
        }

        Ok(())
    }

    async fn set_control_attributes(
        &mut self,
        timestamp: Timestamp,
        display_name: &str,
        avatar_url: Option<&str>,
    ) -> Result<()> {
        let mut actions = self
            .redis_conn
            .bulk_attribute_actions(self.room_id, self.id);

        match &self.participant {
            Participant::User(ref user) => {
                actions
                    .set("kind", ParticipationKind::User)
                    .set(
                        "avatar_url",
                        avatar_url.expect("user must have avatar_url set"),
                    )
                    .set("user_id", user.id)
                    .set("is_room_owner", user.id == self.room.created_by);
            }
            Participant::Guest => {
                actions.set("kind", ParticipationKind::Guest);
            }
            Participant::Sip => {
                actions.set("kind", ParticipationKind::Sip);
            }
            Participant::Recorder => {
                actions.set("kind", ParticipationKind::Recorder);
            }
        }

        actions
            .set("role", self.role)
            .set("hand_is_up", false)
            .set("hand_updated_at", timestamp)
            .set("display_name", display_name)
            .set("joined_at", timestamp)
            .apply(&mut self.redis_conn)
            .await?;

        Ok(())
    }

    /// Fetch all control related data for the given participant id, building a "base" for a participant.
    ///
    /// If the participant is an invisible service (like the recorder) and shouldn't be shown to other participants
    /// this function will return Ok(None)
    async fn build_participant(
        &mut self,
        id: ParticipantId,
    ) -> Result<Option<opentalk_types::signaling::control::Participant>> {
        let mut participant = opentalk_types::signaling::control::Participant {
            id,
            module_data: Default::default(),
        };

        let control_data = ControlState::from_redis(&mut self.redis_conn, self.room_id, id).await?;

        // Do not build participants for invisible services
        if !control_data.participation_kind.is_visible() {
            return Ok(None);
        };

        participant
            .module_data
            .insert(&control_data)
            .expect("Failed to convert ControlData to serde_json::Value");

        Ok(Some(participant))
    }

    #[tracing::instrument(skip_all)]
    async fn handle_exchange_msg(&mut self, msg: ByteString) {
        // Do not handle any messages before the user joined the room
        if let RunnerState::None = &self.state {
            return;
        }

        let namespaced = match serde_json::from_str::<NamespacedEvent<Value>>(&msg) {
            Ok(namespaced) => namespaced,
            Err(e) => {
                log::error!(
                    "Failed to read incoming exchange message, {}",
                    Report::from_error(e)
                );
                return;
            }
        };

        if namespaced.namespace == NAMESPACE {
            let msg = match serde_json::from_value::<exchange::Message>(namespaced.payload) {
                Ok(msg) => msg,
                Err(e) => {
                    log::error!(
                        "Failed to read incoming control exchange message, {}",
                        Report::from_error(e)
                    );
                    return;
                }
            };

            if let Err(e) = self
                .handle_exchange_control_msg(namespaced.timestamp, msg)
                .await
            {
                log::error!(
                    "Failed to handle incoming exchange control msg, {}",
                    Report::from_error(e)
                );
            }
        } else if let RunnerState::Joined = &self.state {
            // Only allow rmq messages outside the control namespace if the participant is fully joined
            match self
                .handle_module_targeted_event(
                    namespaced.namespace,
                    namespaced.timestamp,
                    DynTargetedEvent::ExchangeMessage(namespaced.payload),
                )
                .await
            {
                Ok(actions) => {
                    self.handle_module_requested_actions(namespaced.timestamp, actions)
                        .await
                }
                Err(NoSuchModuleError) => log::warn!("Got invalid exchange message"),
            }
        }
    }

    async fn handle_exchange_control_msg(
        &mut self,
        timestamp: Timestamp,
        msg: exchange::Message,
    ) -> Result<()> {
        log::debug!("Received control message from exchange {:?}", msg);

        match msg {
            exchange::Message::Joined(id) => {
                // Ignore events of self and only if runner is joined
                if self.id == id || !matches!(&self.state, RunnerState::Joined) {
                    return Ok(());
                }

                let mut participant = if let Some(participant) = self.build_participant(id).await? {
                    participant
                } else {
                    return Ok(());
                };

                let actions = self
                    .handle_module_broadcast_event(
                        timestamp,
                        DynBroadcastEvent::ParticipantJoined(&mut participant),
                        false,
                    )
                    .await;

                self.ws_send_control(timestamp, ControlEvent::Joined(participant))
                    .await;

                self.handle_module_requested_actions(timestamp, actions)
                    .await;
            }
            exchange::Message::Left { id, reason } => {
                if self.id == id && reason == Reason::SentToWaitingRoom {
                    // we are sent to the waiting room
                    self.notify_left(id, reason, timestamp).await;
                    self.reenter_waiting_room(timestamp).await?;
                } else if self.id != id && self.state == RunnerState::Joined {
                    // Ignore events of self and ensure runner is in joined state
                    self.notify_left(id, reason, timestamp).await;
                }
            }
            exchange::Message::Update(id) => {
                // Ignore updates of self and only if runner is joined
                if self.id == id || !matches!(&self.state, RunnerState::Joined) {
                    return Ok(());
                }

                let mut participant = if let Some(participant) = self.build_participant(id).await? {
                    participant
                } else {
                    log::warn!("ignoring update of invisible participant");
                    return Ok(());
                };

                let actions = self
                    .handle_module_broadcast_event(
                        timestamp,
                        DynBroadcastEvent::ParticipantUpdated(&mut participant),
                        false,
                    )
                    .await;

                self.ws_send_control(timestamp, ControlEvent::Update(participant))
                    .await;

                self.handle_module_requested_actions(timestamp, actions)
                    .await;
            }
            exchange::Message::Accepted(id) => {
                if self.id != id {
                    log::warn!("Received misrouted control#accepted message");
                    return Ok(());
                }

                if let RunnerState::Waiting {
                    accepted,
                    control_data: _,
                } = &mut self.state
                {
                    if !*accepted {
                        *accepted = true;

                        // Allow the participant to skip the waiting room on next rejoin
                        storage::set_skip_waiting_room_with_expiry(
                            &mut self.redis_conn,
                            self.id,
                            true,
                        )
                        .await?;

                        self.ws
                            .send(Message::Text(
                                serde_json::to_string(&NamespacedEvent {
                                    namespace: moderation::NAMESPACE,
                                    timestamp,
                                    payload: ModerationEvent::Accepted,
                                })
                                .whatever_context::<_, RunnerError>("Failed to send ws message")?
                                .into(),
                            ))
                            .await;
                    }
                }
            }
            exchange::Message::SetModeratorStatus(grant_moderator) => {
                let created_room = if let Participant::User(user) = &self.participant {
                    self.room.created_by == user.id
                } else {
                    false
                };

                if created_room {
                    return Ok(());
                }

                let new_role = if grant_moderator {
                    Role::Moderator
                } else {
                    match &self.participant {
                        Participant::User(_) => Role::User,
                        Participant::Guest | Participant::Sip | Participant::Recorder => {
                            Role::Guest
                        }
                    }
                };

                if self.role == new_role {
                    return Ok(());
                }

                self.role = new_role;

                self.redis_conn
                    .set_attribute(self.room_id, self.id, "role", new_role)
                    .await?;

                let actions = self
                    .handle_module_broadcast_event(
                        timestamp,
                        DynBroadcastEvent::RoleUpdated(new_role),
                        false,
                    )
                    .await;

                self.handle_module_requested_actions(timestamp, actions)
                    .await;

                self.ws_send_control(timestamp, ControlEvent::RoleUpdated { new_role })
                    .await;

                self.exchange_publish_control(timestamp, None, exchange::Message::Update(self.id));
            }
            exchange::Message::ResetRaisedHands { issued_by } => {
                let raised: Option<bool> = self
                    .redis_conn
                    .get_attribute(self.room_id, self.id, "hand_is_up")
                    .await?;
                if matches!(raised, Some(true)) {
                    self.handle_raise_hand_change(timestamp, false).await?;

                    self.ws
                        .send(Message::Text(
                            serde_json::to_string(&NamespacedEvent {
                                namespace: moderation::NAMESPACE,
                                timestamp,
                                payload: ModerationEvent::RaisedHandResetByModerator { issued_by },
                            })
                            .whatever_context::<_, RunnerError>("Failed to send ws message")?
                            .into(),
                        ))
                        .await;
                }
            }
            exchange::Message::EnableRaiseHands { issued_by } => {
                self.ws
                    .send(Message::Text(
                        serde_json::to_string(&NamespacedEvent {
                            namespace: moderation::NAMESPACE,
                            timestamp,
                            payload: ModerationEvent::RaiseHandsEnabled { issued_by },
                        })
                        .whatever_context::<_, RunnerError>("Failed to send ws message")?
                        .into(),
                    ))
                    .await;
            }
            exchange::Message::DisableRaiseHands { issued_by } => {
                let raised: Option<bool> = self
                    .redis_conn
                    .get_attribute(self.room_id, self.id, "hand_is_up")
                    .await?;
                if matches!(raised, Some(true)) {
                    self.handle_raise_hand_change(timestamp, false).await?;
                }

                self.ws
                    .send(Message::Text(
                        serde_json::to_string(&NamespacedEvent {
                            namespace: moderation::NAMESPACE,
                            timestamp,
                            payload: ModerationEvent::RaiseHandsDisabled { issued_by },
                        })
                        .whatever_context::<_, RunnerError>("Failed to send ws message")?
                        .into(),
                    ))
                    .await;
            }
            exchange::Message::RoomDeleted => {
                self.ws_send_control(timestamp, ControlEvent::RoomDeleted)
                    .await;
                self.ws.close(CloseCode::Normal).await;
            }
        }

        Ok(())
    }

    async fn reenter_waiting_room(&mut self, timestamp: Timestamp) -> Result<(), RunnerError> {
        ensure!(
            self.state == RunnerState::Joined,
            InvalidStateSnafu {
                message: "Can only reenter the waiting room if already joined",
            }
        );

        self.redis_conn
            .set_attribute(self.room_id, self.id, "left_at", Timestamp::now())
            .await?;

        let actions = self
            .handle_module_broadcast_event(timestamp, DynBroadcastEvent::Leaving, false)
            .await;

        self.handle_module_requested_actions(timestamp, actions)
            .await;

        // resuming ensures that we can reuse the same participant ID
        self.resuming = true;
        moderation::storage::waiting_room_add(
            &mut self.redis_conn,
            self.room_id.room_id(),
            self.id,
        )
        .await?;

        let control_data =
            ControlState::from_redis(&mut self.redis_conn, self.room_id, self.id).await?;

        self.state = RunnerState::Waiting {
            accepted: false,
            control_data,
        };

        self.ws
            .send(Message::Text(
                serde_json::to_string(&NamespacedEvent {
                    namespace: moderation::NAMESPACE,
                    timestamp,
                    payload: ModerationEvent::InWaitingRoom,
                })
                .whatever_context::<_, RunnerError>("Failed to send")?
                .into(),
            ))
            .await;

        self.exchange_publish(
            control::exchange::global_room_all_participants(self.room_id.room_id()),
            serde_json::to_string(&NamespacedEvent {
                namespace: moderation::NAMESPACE,
                timestamp,
                payload: moderation::exchange::Message::JoinedWaitingRoom(self.id),
            })
            .expect("Failed to convert namespaced to json"),
        );

        Ok(())
    }

    /// Send a control message via the message exchange
    ///
    /// If recipient is `None` the message is sent to all inside the room
    fn exchange_publish_control(
        &mut self,
        timestamp: Timestamp,
        recipient: Option<ParticipantId>,
        message: exchange::Message,
    ) {
        let message = NamespacedEvent {
            namespace: NAMESPACE,
            timestamp,
            payload: message,
        };

        let routing_key = if let Some(recipient) = recipient {
            exchange::current_room_by_participant_id(self.room_id, recipient)
        } else {
            exchange::current_room_all_participants(self.room_id)
        };

        self.exchange_publish(
            routing_key,
            serde_json::to_string(&message).expect("Failed to convert namespaced to json"),
        );
    }

    fn exchange_publish(&mut self, routing_key: String, message: String) {
        if let Err(e) = self.exchange_handle.publish(routing_key, message) {
            log::warn!(
                "Failed to publish message to exchange, {}",
                Report::from_error(e)
            );
            self.exit = true;
        }
    }

    /// Dispatch owned event to a single module
    async fn handle_module_targeted_event(
        &mut self,
        module: &str,
        timestamp: Timestamp,
        dyn_event: DynTargetedEvent,
    ) -> Result<ModuleRequestedActions, NoSuchModuleError> {
        let mut ws_messages = vec![];
        let mut exchange_publish = vec![];
        let mut invalidate_data = false;
        let mut exit = None;

        let ctx = DynEventCtx {
            id: self.id,
            role: self.role,
            timestamp,
            ws_messages: &mut ws_messages,
            exchange_publish: &mut exchange_publish,
            redis_conn: &mut self.redis_conn,
            events: &mut self.events,
            invalidate_data: &mut invalidate_data,
            exit: &mut exit,
            metrics: self.metrics.clone(),
        };

        self.modules
            .on_event_targeted(ctx, module, dyn_event)
            .await?;

        Ok(ModuleRequestedActions {
            ws_messages,
            exchange_publish,
            invalidate_data,
            exit,
        })
    }

    /// Dispatch copyable event to all modules
    async fn handle_module_broadcast_event(
        &mut self,
        timestamp: Timestamp,
        dyn_event: DynBroadcastEvent<'_>,
        mut invalidate_data: bool,
    ) -> ModuleRequestedActions {
        let mut ws_messages = vec![];
        let mut exchange_publish = vec![];
        let mut exit = None;

        let ctx = DynEventCtx {
            id: self.id,
            role: self.role,
            timestamp,
            ws_messages: &mut ws_messages,
            exchange_publish: &mut exchange_publish,
            redis_conn: &mut self.redis_conn,
            events: &mut self.events,
            invalidate_data: &mut invalidate_data,
            exit: &mut exit,
            metrics: self.metrics.clone(),
        };

        self.modules.on_event_broadcast(ctx, dyn_event).await;

        ModuleRequestedActions {
            ws_messages,
            exchange_publish,
            invalidate_data,
            exit,
        }
    }

    /// Modules can request certain actions via the module context (e.g send websocket msg)
    /// these are executed here
    async fn handle_module_requested_actions(
        &mut self,
        timestamp: Timestamp,
        ModuleRequestedActions {
            ws_messages,
            exchange_publish,
            invalidate_data,
            exit,
        }: ModuleRequestedActions,
    ) {
        for ws_message in ws_messages {
            self.ws.send(ws_message).await;
        }

        for publish in exchange_publish {
            self.exchange_publish(publish.routing_key, publish.message);
        }

        if invalidate_data {
            self.exchange_publish_control(timestamp, None, exchange::Message::Update(self.id));
        }

        if let Some(exit) = exit {
            self.ws.close(exit).await;
        }
    }

    async fn ws_send_control_error(&mut self, timestamp: Timestamp, error: control_event::Error) {
        self.ws_send_control(timestamp, ControlEvent::Error(error))
            .await;
    }

    async fn ws_send_control(&mut self, timestamp: Timestamp, payload: ControlEvent) {
        self.ws
            .send(Message::Text(
                serde_json::to_string(&NamespacedEvent {
                    namespace: NAMESPACE,
                    timestamp,
                    payload,
                })
                .expect("Failed to convert namespaced to json")
                .into(),
            ))
            .await;
    }

    async fn notify_left(&mut self, id: ParticipantId, reason: Reason, timestamp: Timestamp) {
        let actions: ModuleRequestedActions = self
            .handle_module_broadcast_event(timestamp, DynBroadcastEvent::ParticipantLeft(id), false)
            .await;

        if self.id != id {
            self.ws_send_control(
                timestamp,
                ControlEvent::Left {
                    id: AssociatedParticipant { id },
                    reason,
                },
            )
            .await;
        }

        self.handle_module_requested_actions(timestamp, actions)
            .await;
    }

    async fn username_or(&self, join_display_name: Option<String>) -> String {
        let join_display_name = join_display_name.unwrap_or_default();

        match &self.participant {
            Participant::User(user) => {
                // Enforce the auto-generated display name if display name editing is prohibited
                let settings = self.settings.load();
                let user_display_name = if settings.endpoints.disallow_custom_display_name {
                    user.display_name.clone()
                } else {
                    join_display_name.clone()
                };

                trim_display_name(user_display_name)
            }
            Participant::Guest => trim_display_name(join_display_name),
            Participant::Recorder => join_display_name,
            Participant::Sip => {
                if let Some(call_in) = self.settings.load().call_in.as_ref() {
                    call_in::display_name(&self.db, call_in, self.room.tenant_id, join_display_name)
                        .await
                } else {
                    trim_display_name(join_display_name)
                }
            }
        }
    }

    async fn avatar_url(&self) -> Option<String> {
        match &self.participant {
            Participant::User(user) => {
                let settings = self.settings.load();
                Some(format!(
                    "{}{:x}",
                    settings.avatar.libravatar_url,
                    md5::compute(&user.email)
                ))
            }
            Participant::Guest | Participant::Recorder | Participant::Sip => None,
        }
    }
}

#[must_use]
struct ModuleRequestedActions {
    ws_messages: Vec<Message>,
    exchange_publish: Vec<ExchangePublish>,
    invalidate_data: bool,
    exit: Option<CloseCode>,
}

struct Ws {
    to_actor: Addr<WebSocketActor>,
    from_actor: mpsc::UnboundedReceiver<RunnerMessage>,

    state: State,
}

enum State {
    Open,
    Closed,
    Error,
}

impl Ws {
    /// Send message via websocket
    async fn send(&mut self, message: Message) {
        if let State::Open = self.state {
            log::trace!("Send message to websocket: {:?}", message);

            if let Err(e) = self.to_actor.send(WsCommand::Ws(message)).await {
                log::error!(
                    "Failed to send websocket message, {}",
                    Report::from_error(e)
                );
                self.state = State::Error;
            }
        } else {
            log::warn!("Tried to send websocket message on closed or error'd websocket");
        }
    }

    /// Close the websocket connection if needed
    async fn close(&mut self, code: CloseCode) {
        if !matches!(self.state, State::Open) {
            return;
        }

        let reason = CloseReason {
            code,
            description: None,
        };

        log::debug!("closing websocket with code {:?}", code);

        self.state = State::Closed;
        if let Err(e) = self.to_actor.send(WsCommand::Close(reason)).await {
            log::error!("Failed to close websocket, {}", Report::from_error(e));
            self.state = State::Error;
        }
    }

    /// Receive a message from the websocket
    ///
    /// Sends a health check ping message every WS_TIMEOUT.
    async fn receive(&mut self) -> Option<RunnerMessage> {
        match self.from_actor.recv().await {
            Some(msg) => Some(msg),
            None => {
                self.state = State::Closed;
                None
            }
        }
    }
}
