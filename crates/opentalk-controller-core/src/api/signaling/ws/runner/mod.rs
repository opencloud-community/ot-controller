// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{BTreeMap, BTreeSet},
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
use futures::{Future, stream::SelectAll};
use kustos::Authz;
use log::log_enabled;
use opentalk_controller_service::{
    email_to_libravatar_url,
    signaling::{
        resumption::ResumptionTokenKeepAlive,
        storage::{SignalingStorageError, SignalingStorageProvider},
        ws_modules::{
            breakout,
            echo::Echo,
            moderation::{self, ModerationStorageProvider},
        },
    },
};
use opentalk_controller_settings::SettingsProvider;
use opentalk_database::{Db, DbConnection};
use opentalk_db_storage::{
    events::EventInvite, rooms::Room, tariffs::Tariff, users::User, utils::build_event_info,
};
use opentalk_signaling_core::{
    AnyStream, ExchangeHandle, LockError, ObjectStorage, Participant, RoomLockingProvider as _,
    RunnerId, SignalingMetrics, SignalingModule, SignalingModuleError, SignalingRoomId,
    SubscriberHandle, VolatileStorage,
    control::{
        self, ControlStateExt as _, ControlStorageProvider, MODULE_ID, exchange,
        storage::{
            AVATAR_URL, AttributeActions, BREAKOUT_ROOM, ControlStorageParticipantAttributes,
            DISPLAY_NAME, GlobalRoomAttributeId, HAND_IS_UP, HAND_UPDATED_AT, IS_PRESENT,
            IS_ROOM_OWNER, JOINED_AT, KIND, LEFT_AT, LocalRoomAttributeId, ROLE, USER_ID,
        },
    },
};
use opentalk_types_common::{
    features::FeatureId,
    modules::ModuleId,
    rooms::{BreakoutRoomId, RoomId},
    tariffs::{QuotaType, TariffResource},
    time::Timestamp,
    users::{DisplayName, UserId, UserInfo},
};
use opentalk_types_signaling::{
    AssociatedParticipant, LeaveReason, ModuleData, NamespacedCommand, ParticipantId,
    ParticipationKind, Role, TargetParticipant,
};
use opentalk_types_signaling_control::{
    command::ControlCommand,
    event::{
        self as control_event, ControlEvent, JoinBlockedReason, JoinSuccess, Left, RoleUpdated,
    },
    room::RoomInfo,
    state::ControlState,
};
use opentalk_types_signaling_moderation::event::{
    ModerationEvent, RaiseHandsDisabled, RaiseHandsEnabled, RaisedHandResetByModerator,
};
use serde_json::Value;
use snafu::{Report, ResultExt, Snafu, ensure, whatever};
use tokio::{
    sync::{broadcast, mpsc},
    time::{interval, sleep},
};
use tokio_stream::StreamExt;

use super::{
    CleanupScope, DestroyContext, ExchangeBinding, ExchangePublish, NamespacedEvent, RunnerMessage,
    actor::WebSocketActor,
    modules::{DynBroadcastEvent, DynEventCtx, DynTargetedEvent, Modules, NoSuchModuleError},
};
use crate::api::signaling::ws::actor::WsCommand;

mod call_in;

const GRACE_PERIOD_DURATION: u64 = 60;

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
    RoomLock {
        source: LockError,
    },

    #[snafu(context(false))]
    Redis {
        source: redis::RedisError,
    },

    #[snafu(context(false))]
    Storage {
        source: SignalingStorageError,
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
    runner_id: RunnerId,
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
    pub(super) volatile: VolatileStorage,
    pub(super) exchange_handle: ExchangeHandle,
    resumption_keep_alive: ResumptionTokenKeepAlive,
}

impl Builder {
    /// Abort the building process and destroy all already built modules
    #[tracing::instrument(skip(self))]
    pub async fn abort(mut self) {
        let ctx = DestroyContext {
            volatile: &mut self.volatile,
            // We haven't joined yet
            cleanup_scope: CleanupScope::None,
        };

        self.modules.destroy(ctx).await
    }

    /// Build to runner from the data inside the builder and provided websocket
    #[tracing::instrument(err, skip_all)]
    pub async fn build(
        mut self,
        to_ws_actor: Addr<WebSocketActor>,
        from_ws_actor: mpsc::UnboundedReceiver<RunnerMessage>,
        shutdown_sig: broadcast::Receiver<()>,
        settings_provider: SettingsProvider,
    ) -> Result<Runner> {
        self.volatile
            .signaling_storage()
            .acquire_participant_id(self.id, self.runner_id)
            .await?;

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
            .set_initial(self.volatile.signaling_storage())
            .await?;

        if self.room.e2e_encryption {
            self.modules
                .get_module_features_mut()
                .remove(&opentalk_types_signaling_recording::MODULE_ID);
            self.modules
                .get_module_features_mut()
                .remove(&opentalk_types_signaling_recording_service::MODULE_ID);
            self.modules
                .get_module_features_mut()
                .entry(ModuleId::default())
                .and_modify(|f| {
                    f.remove(&opentalk_types_common::features::CALL_IN_FEATURE_ID);
                });
        }

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
            volatile: self.volatile,
            exchange_handle: self.exchange_handle,
            subscriber_handle,
            resumption_keep_alive: self.resumption_keep_alive,
            shutdown_sig,
            exit: false,
            leave_reason: LeaveReason::Quit,
            settings_provider,
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
    runner_id: RunnerId,

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

    volatile: VolatileStorage,

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

    /// The reason why a user disconnected
    leave_reason: LeaveReason,

    /// Shared settings of the running program
    settings_provider: SettingsProvider,

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

async fn get_adhoc_role(
    volatile: &mut VolatileStorage,
    room_id: RoomId,
    participant_id: ParticipantId,
) -> Result<Option<Role>> {
    volatile
        .control_storage()
        .get_global_attribute(participant_id, room_id, ROLE)
        .await
        .whatever_context::<_, RunnerError>("Failed to get role from volatile storage")
}

impl Runner {
    #[allow(clippy::too_many_arguments)]
    pub async fn builder(
        runner_id: RunnerId,
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
        mut volatile: VolatileStorage,
        exchange_handle: ExchangeHandle,
        resumption_keep_alive: ResumptionTokenKeepAlive,
    ) -> Result<Builder> {
        let role = match get_adhoc_role(&mut volatile, room.id, id).await? {
            Some(adhoc_role) => adhoc_role,
            None => get_participant_role(&mut db.get_conn().await?, &participant, &room).await?,
        };

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
            volatile,
            exchange_handle,
            resumption_keep_alive,
        })
    }

    /// Destroys the runner and all associated resources
    #[tracing::instrument(skip(self), fields(id = %self.id))]
    pub async fn destroy(mut self, close_ws: bool, grace_period: bool) {
        let mut destroy_start_time = Instant::now();
        let mut encountered_error = false;
        let mut cleanup_scope = CleanupScope::None;

        if let RunnerState::Joined | RunnerState::Waiting { .. } = &self.state {
            // The retry/wait_time values are set extra high
            // since a lot of operations are being done while holding the lock

            let room_guard = match self.volatile.room_locking().lock_room(self.room_id).await {
                Ok(guard) => guard,
                Err(e @ LockError::Locked) => {
                    log::error!(
                        "Failed to acquire r3dlock, contention too high, {}",
                        Report::from_error(e)
                    );

                    self.metrics
                        .record_destroy_time(destroy_start_time.elapsed().as_secs_f64(), false);

                    return;
                }
                Err(e) => {
                    log::error!("Failed to acquire r3dlock, {}", Report::from_error(e));
                    // There is a problem when accessing redis which could
                    // mean either the network or redis is broken.
                    // Both cases cannot be handled here, abort the cleanup

                    self.metrics
                        .record_destroy_time(destroy_start_time.elapsed().as_secs_f64(), false);

                    return;
                }
            };

            if let RunnerState::Joined = &self.state {
                // first check if the list of joined participant is empty
                let res: Result<(), _> = self
                    .volatile
                    .control_storage()
                    .bulk_attribute_actions(
                        AttributeActions::new(self.room_id, self.id)
                            .set_global(IS_PRESENT, false)
                            .set_global(BREAKOUT_ROOM, None::<BreakoutRoomId>)
                            .set_local(LEFT_AT, Timestamp::now()),
                    )
                    .await;
                if let Err(e) = res {
                    log::error!(
                        "failed to mark participant as left, {}",
                        Report::from_error(e)
                    );
                    encountered_error = true;
                }
            } else if let RunnerState::Waiting { .. } = &self.state {
                if let Err(e) = self
                    .volatile
                    .moderation_storage()
                    .waiting_room_remove_participant(self.room_id.room_id(), self.id)
                    .await
                {
                    log::error!(
                        "failed to remove participant from waiting_room list, {:?}",
                        e
                    );
                    encountered_error = true;
                }
                if let Err(e) = self
                    .volatile
                    .moderation_storage()
                    .waiting_room_accepted_remove_participant(self.room_id.room_id(), self.id)
                    .await
                {
                    log::error!(
                        "failed to remove participant from waiting_room_accepted list, {:?}",
                        e
                    );
                    encountered_error = true;
                }
            };

            if let Err(err) = self
                .volatile
                .control_storage()
                .decrement_participant_count(self.room.id)
                .await
            {
                log::error!(
                    "Failed to decrement participant count, {}",
                    Report::from_error(err)
                );
                encountered_error = true;
            }

            cleanup_scope = match self.get_cleanup_scope().await {
                Ok(cleanup_scope) => cleanup_scope,
                Err(e) => {
                    log::error!(
                        "Failed to get cleanup scope after grace period ended: {}",
                        Report::from_error(e)
                    );

                    log::error!("This may result in stale state for room {}", self.room_id);

                    encountered_error = true;
                    CleanupScope::None
                }
            };

            self.metrics.record_participant_left(
                self.room_id.room_id(),
                &self.participant,
                self.id,
            );

            if let Err(e) = self.volatile.room_locking().unlock_room(room_guard).await {
                log::error!("Failed to unlock room , {}", Report::from_error(e));
                encountered_error = true;
            }

            if cleanup_scope.keep_room() {
                match &self.state {
                    RunnerState::None => unreachable!("state was checked before"),
                    RunnerState::Waiting { .. } => {
                        self.exchange_publish(
                            exchange::global_room_all_participants(self.room_id.room_id()),
                            serde_json::to_string(&NamespacedEvent {
                                module: opentalk_types_signaling_moderation::MODULE_ID,
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
                                    reason: self.leave_reason,
                                },
                            );
                        }
                    }
                }
            }
        }

        // release participant id
        match self
            .volatile
            .signaling_storage()
            .release_participant_id(self.id)
            .await
        {
            Ok(Some(runner)) if runner == self.runner_id => {}
            Ok(Some(_)) => {
                log::warn!("removed runner id does not match the id of the runner");
            }
            Ok(None) => log::warn!("attempted to release nonexisting participant lock"),
            Err(err) => {
                log::error!(
                    "failed to remove participant id, {}",
                    Report::from_error(&err)
                );
                encountered_error = true;
            }
        }

        // If a Close frame is received from the websocket actor, manually return a close command
        if close_ws {
            self.ws.close(CloseCode::Normal).await;
        }

        if cleanup_scope.keep_room() {
            self.destroy_modules(cleanup_scope).await;

            self.metrics.record_destroy_time(
                destroy_start_time.elapsed().as_secs_f64(),
                !encountered_error,
            );

            log::debug!("Exiting room without cleaning up for room {}", self.room_id);

            return;
        }

        if let RunnerState::None = &self.state {
            // Early return if the runner never joined the conference
            return;
        }

        if grace_period {
            // At this point, the runner is the last participant in the room. Keep the room alive for a grace period
            if let Err(e) = self.wait_grace_period(&mut cleanup_scope).await {
                log::error!(
                    "Encountered error during grace period, {}",
                    Report::from_error(e)
                );

                self.metrics
                    .record_destroy_time(destroy_start_time.elapsed().as_secs_f64(), false);

                return;
            }

            // reset the destroy start time
            destroy_start_time = Instant::now();
            if cleanup_scope.keep_room() {
                self.destroy_modules(cleanup_scope).await;

                self.metrics.record_destroy_time(
                    destroy_start_time.elapsed().as_secs_f64(),
                    !encountered_error,
                );

                return;
            }
        }

        // acquire room lock
        let room_guard = match self.volatile.room_locking().lock_room(self.room_id).await {
            Ok(guard) => guard,
            Err(e @ LockError::Locked) => {
                log::error!(
                    "Failed to acquire r3dlock, contention too high, {}",
                    Report::from_error(e)
                );

                self.metrics
                    .record_destroy_time(destroy_start_time.elapsed().as_secs_f64(), false);

                return;
            }
            Err(e) => {
                log::error!("Failed to acquire r3dlock, {}", Report::from_error(e));
                // There is a problem when accessing redis which could
                // mean either the network or redis is broken.
                // Both cases cannot be handled here, abort the cleanup

                self.metrics
                    .record_destroy_time(destroy_start_time.elapsed().as_secs_f64(), false);

                return;
            }
        };

        // Get the cleanup scope again. We can't ensure that no user joined after the end of the grace period
        // and before acquiring the room lock:
        //
        //                 <users might join here>
        //                        |-----|
        // |-----grace period----|
        //                               |-----room lock----|
        cleanup_scope = match self.get_cleanup_scope().await {
            Ok(cleanup_scope) => cleanup_scope,
            Err(e) => {
                log::error!(
                    "Failed to get cleanup scope after grace period ended: {}",
                    Report::from_error(e)
                );

                log::error!("This may result in stale state for room {}", self.room_id);

                encountered_error = true;

                CleanupScope::None
            }
        };

        self.cleanup_routine(cleanup_scope, &mut encountered_error)
            .await;

        if let Err(e) = self.volatile.room_locking().unlock_room(room_guard).await {
            log::error!("Failed to unlock room , {}", Report::from_error(e));
        }

        self.metrics.record_destroy_time(
            destroy_start_time.elapsed().as_secs_f64(),
            !encountered_error,
        );

        log::debug!("Finished room cleanup for {}", self.room_id);
    }

    async fn cleanup_routine(&mut self, cleanup_scope: CleanupScope, encountered_error: &mut bool) {
        self.destroy_modules(cleanup_scope).await;

        match cleanup_scope {
            CleanupScope::None => {
                log::debug!("Destroying nothing");
            }
            CleanupScope::Local => {
                log::debug!("Destroying breakout room");

                if let Err(e) =
                    cleanup_redis_keys_for_signaling_room(&mut self.volatile, self.room_id).await
                {
                    log::error!(
                        "Failed to remove all control attributes, {}",
                        Report::from_error(e)
                    );

                    *encountered_error = true;
                }

                self.metrics.increment_destroyed_breakout_rooms_count();
            }
            CleanupScope::Global => {
                log::debug!("Destroying conference room");
                if let Err(e) =
                    cleanup_redis_keys_for_signaling_room(&mut self.volatile, self.room_id).await
                {
                    log::error!(
                        "Failed to remove all control attributes, {}",
                        Report::from_error(e)
                    );

                    *encountered_error = true;
                }

                if self.room_id.breakout_room_id().is_some() {
                    self.metrics.increment_destroyed_breakout_rooms_count();

                    // Cleanup the signaling room keys for the main room
                    if let Err(e) = cleanup_redis_keys_for_signaling_room(
                        &mut self.volatile,
                        SignalingRoomId::new(self.room_id.room_id(), None),
                    )
                    .await
                    {
                        log::error!("failed to cleanup main room {}", Report::from_error(e));

                        *encountered_error = true;
                    }
                }

                if !self
                    .volatile
                    .control_storage()
                    .is_room_alive(self.room_id.room_id())
                    .await
                    // Continue with cleanup if we receive an error on the redis call
                    .unwrap_or(true)
                {
                    return;
                }

                if let Err(e) = self.cleanup_redis_for_global_room().await {
                    log::error!(
                        "failed to cleanup conference room, {}",
                        Report::from_error(e)
                    );

                    *encountered_error = true;
                }

                self.metrics
                    .record_room_destroyed_metrics(self.room_id.room_id());
            }
        }
    }

    /// Determine what state of the room should be cleaned up
    ///
    /// Note: This must be called after the current participant has left the room and while the room lock is held
    async fn get_cleanup_scope(&mut self) -> Result<CleanupScope, SignalingModuleError> {
        // Counts for all participants from the current room, waiting room and breakout rooms
        let global_room_participants: usize = self
            .volatile
            .control_storage()
            .get_participant_count(self.room_id.room_id())
            .await?
            .unwrap_or(0)
            .max(0) as usize;

        let current_room_empty = self
            .volatile
            .control_storage()
            .participants_all_left(self.room_id)
            .await?;

        log::debug!(
            "Participant state while determining cleanup scope ( current room empty: {current_room_empty}, global room: {global_room_participants})"
        );

        if global_room_participants == 0 {
            // The main room, waiting room and all breakout rooms are empty
            return Ok(CleanupScope::Global);
        }

        if current_room_empty && self.room_id.breakout_room_id().is_some() {
            // Participants exist in the main room or in the waiting room, but the local state of this room can be
            // cleaned up
            return Ok(CleanupScope::Local);
        }

        // There are still participants in either a breakout or waiting room, can't clean up the global room yet
        Ok(CleanupScope::None)
    }

    /// Determine what state should be cleaned up when a participant joined the conference after the grace period has
    /// started
    fn get_cleanup_scope_after_join(&self, join_event: JoinEvent) -> CleanupScope {
        if self.room_id.breakout_room_id().is_none() {
            // If this runner is the main room, the room destruction is canceled when a participant joined during the
            // grace period
            return CleanupScope::None;
        }

        // This runner is a breakout room, we might still have to do a partial cleanup, even after someone joined the
        // conference
        match join_event {
            JoinEvent::WaitingRoom => CleanupScope::Local,
            JoinEvent::Room(signaling_room_id) => {
                if signaling_room_id == self.room_id {
                    // Someone joined this specific breakout room, cancel the cleanup
                    CleanupScope::None
                } else {
                    CleanupScope::Local
                }
            }
        }
    }

    /// Keeps the room alive for a grace period
    ///
    /// The grace period can be canceled early when another participant joins the conference and meets the conditions
    /// to abort the cleanup for this specific room.
    ///
    /// Updates the `cleanup_scope` depending on the join event.
    async fn wait_grace_period(
        &mut self,
        cleanup_scope: &mut CleanupScope,
    ) -> Result<(), snafu::Whatever> {
        log::debug!(
            "Entering room destruction grace period for {}",
            self.room_id
        );
        let mut grace_period = Box::pin(tokio::time::sleep(Duration::from_secs(
            GRACE_PERIOD_DURATION,
        )));

        loop {
            tokio::select! {
                msg = self.subscriber_handle.receive() => {
                    match msg {
                        Some(msg) => {
                            if let Some(join_event) = self.has_participant_joined(msg) {
                                *cleanup_scope = self.get_cleanup_scope_after_join(join_event);
                            }

                            if cleanup_scope == &CleanupScope::None {
                                // A participant joined this room, cancel the destruction
                                break;
                            }
                        },
                        None => {
                            whatever!("Exchange handle dropped")
                        },
                    }
                }
                _ = self.shutdown_sig.recv() => {
                    // Received shutdown signal, abort grace period and destroy global room
                    *cleanup_scope = CleanupScope::Global;
                    break;
                }
                _ = &mut grace_period => {
                    log::debug!("Idle timeout reached for {}", self.room_id);
                    break;
                }
            }
        }

        if log_enabled!(log::Level::Debug) {
            let remaining_time = grace_period
                .deadline()
                .duration_since(tokio::time::Instant::now());

            match remaining_time.as_secs() {
                0 => log::debug!(
                    "Finished full grace period, cleanup scope is {:?} for {}",
                    cleanup_scope,
                    self.room_id
                ),
                remaining => log::debug!(
                    "Finished grace period early with {} second left, cleanup scope is {:?} for {}",
                    remaining,
                    cleanup_scope,
                    self.room_id
                ),
            }
        }

        Ok(())
    }

    async fn destroy_modules(&mut self, cleanup_scope: CleanupScope) {
        log::debug!(
            "Destroying modules with cleanup scope {:?} for {}",
            cleanup_scope,
            self.room_id
        );

        let ctx = DestroyContext {
            volatile: &mut self.volatile,
            cleanup_scope,
        };

        self.modules.destroy(ctx).await;
    }

    /// Remove all room and control module related redis keys that are used across all 'sub'-rooms. This must only be
    /// called once the main and all breakout rooms are empty.
    async fn cleanup_redis_for_global_room(&mut self) -> Result<()> {
        log::debug!("Cleanup up global room for {}", self.room_id);
        self.volatile
            .control_storage()
            .delete_participant_count(self.room.id)
            .await?;
        self.volatile
            .control_storage()
            .delete_tariff(self.room.id)
            .await?;
        self.volatile
            .control_storage()
            .delete_event(self.room.id)
            .await?;
        self.volatile
            .control_storage()
            .delete_creator(self.room.id)
            .await?;
        self.volatile
            .control_storage()
            .delete_room_alive(self.room.id)
            .await?;

        Ok(())
    }

    /// Runs the runner until the peer closes its websocket connection or a fatal error occurs.
    pub async fn run(mut self) {
        let mut manual_close_ws = false;
        let mut grace_period = true;

        // Set default `skip_waiting_room` key value with the default expiration time
        _ = self
            .volatile
            .control_storage()
            .set_skip_waiting_room_with_expiry_nx(self.id, false)
            .await;
        let mut skip_waiting_room_refresh_interval = interval(Duration::from_secs(
            opentalk_signaling_core::control::storage::SKIP_WAITING_ROOM_KEY_REFRESH_INTERVAL,
        ));

        while matches!(self.ws.state, State::Open) {
            if self.exit && matches!(self.ws.state, State::Open) {
                // This case handles exit on errors unrelated to websocket or controller shutdown
                self.ws.close(CloseCode::Abnormal).await;
            }

            tokio::select! {
                res = self.ws.receive() => {
                    match res {
                        Some(RunnerMessage::Timeout) => self.leave_reason = LeaveReason::Timeout,
                        Some(RunnerMessage::Message(Message::Close(_))) => {
                            self.leave_reason = LeaveReason::Quit;
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
                Some((module_id, any)) = self.events.next() => {
                    let timestamp = Timestamp::now();
                    let actions = self.handle_module_targeted_event(&module_id, timestamp, DynTargetedEvent::Ext(any))
                        .await
                        .expect("Should not get events from unknown modules");

                    self.handle_module_requested_actions(timestamp, actions).await;
                }
                _ = skip_waiting_room_refresh_interval.tick() => {
                    _ = self.volatile.control_storage().reset_skip_waiting_room_expiry(
                        self.id,
                    )
                    .await;
                }
                _ = &mut self.time_limit_future => {
                    self.ws_send_control(Timestamp::now(), ControlEvent::TimeLimitQuotaElapsed).await;
                    self.ws.close(CloseCode::Normal).await;
                    grace_period = false;
                    break;
                }
                _ = self.shutdown_sig.recv() => {
                    self.ws.close(CloseCode::Away).await;
                    grace_period = false;
                    break;
                }
                _ = self.resumption_keep_alive.wait() => {
                    match self.resumption_keep_alive.refresh(self.volatile.signaling_storage()).await {
                        Ok(_) => {},
                        Err(SignalingStorageError::ResumptionTokenAlreadyUsed) => {
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

        self.destroy(manual_close_ws, grace_period).await;
    }

    #[tracing::instrument(skip(self, message), fields(id = %self.id))]
    async fn handle_ws_message(&mut self, message: Message) {
        log::trace!("Received websocket message {:?}", message);

        let value: Result<NamespacedCommand<Value>, _> = match message {
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

        if namespaced.module == MODULE_ID {
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
        } else if matches!(&self.state, RunnerState::Joined) || namespaced.module == Echo::NAMESPACE
        {
            match self
                .handle_module_targeted_event(
                    &namespaced.module,
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
                    log::warn!(
                        "received message with invalid namespace: {}",
                        namespaced.module
                    );
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

                self.metrics.record_participant_joined(
                    self.room_id.room_id(),
                    &self.participant,
                    self.id,
                );

                // Allow moderators, invisible services, and already accepted participants to skip the waiting room
                let can_skip_waiting_room: bool = self
                    .volatile
                    .control_storage()
                    .get_skip_waiting_room(self.id)
                    .await?;

                let skip_waiting_room = matches!(self.role, Role::Moderator)
                    || control_data.participation_kind.visibility().is_hidden()
                    || can_skip_waiting_room;

                let waiting_room_enabled = self
                    .volatile
                    .moderation_storage()
                    .init_waiting_room_enabled(self.room_id.room_id(), self.room.waiting_room)
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
                                module: opentalk_types_signaling_moderation::MODULE_ID,
                                timestamp,
                                payload: moderation::exchange::Message::LeftWaitingRoom(self.id),
                            })
                            .expect("Failed to convert namespaced to json"),
                        );

                        self.volatile
                            .moderation_storage()
                            .waiting_room_accepted_remove_participant(
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
                if !self
                    .volatile
                    .moderation_storage()
                    .is_raise_hands_enabled(self.room.id)
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
        join_display_name: Option<DisplayName>,
        timestamp: Timestamp,
    ) -> Result<ControlState, RunnerError> {
        let display_name = self.username_or(join_display_name).await;
        let avatar_url = self.avatar_url().await;

        if display_name.is_empty() || display_name.len() > 100 {
            return InvalidDisplayNameSnafu.fail();
        }

        let left_at: Option<Timestamp> = self
            .volatile
            .control_storage()
            .get_local_attribute(self.id, self.room_id, LEFT_AT)
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
            .volatile
            .control_storage()
            .get_global_attribute(target, self.room_id.room_id(), ROLE)
            .await?;

        let is_moderator = matches!(role, Some(Role::Moderator));

        if is_moderator == grant {
            self.ws_send_control_error(timestamp, control_event::Error::NothingToDo)
                .await;

            return Ok(());
        }

        let user_id: Option<UserId> = self
            .volatile
            .control_storage()
            .get_local_attribute(target, self.room_id, USER_ID)
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

        let message_to_initiator = if grant {
            ControlEvent::ModeratorRoleGranted(TargetParticipant { target })
        } else {
            ControlEvent::ModeratorRoleRevoked(TargetParticipant { target })
        };

        self.ws_send_control(timestamp, message_to_initiator).await;

        Ok(())
    }

    async fn handle_raise_hand_change(
        &mut self,
        timestamp: Timestamp,
        hand_raised: bool,
    ) -> Result<()> {
        self.volatile
            .control_storage()
            .bulk_attribute_actions::<()>(
                AttributeActions::new(self.room_id, self.id)
                    .set_local(HAND_IS_UP, hand_raised)
                    .set_local(HAND_UPDATED_AT, timestamp),
            )
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
            .volatile
            .control_storage()
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
            .volatile
            .control_storage()
            .try_init_tariff(self.room.id, tariff)
            .await?;

        if self.role == Role::Moderator && !self.room_has_moderator_besides_me().await? {
            self.volatile
                .control_storage()
                .increment_participant_count(self.room.id)
                .await?;
            return Ok(ControlFlow::Continue(tariff));
        }

        if let Some(participant_limit) = tariff.quota(&QuotaType::RoomParticipantLimit) {
            if let Some(count) = self
                .volatile
                .control_storage()
                .get_participant_count(self.room.id)
                .await?
            {
                if count >= participant_limit as isize {
                    return Ok(ControlFlow::Break(
                        JoinBlockedReason::ParticipantLimitReached,
                    ));
                }
            }
        }

        self.volatile
            .control_storage()
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

        let guard = self.volatile.room_locking().lock_room(self.room_id).await?;

        match self.enforce_tariff(tariff).await {
            Ok(ControlFlow::Continue(_)) => { /* continue */ }
            Ok(ControlFlow::Break(reason)) => {
                self.volatile.room_locking().unlock_room(guard).await?;

                self.ws_send_control(Timestamp::now(), ControlEvent::JoinBlocked(reason))
                    .await;

                return Ok(());
            }
            Err(e) => {
                self.volatile.room_locking().unlock_room(guard).await?;

                return Err(e);
            }
        };

        let res = self
            .volatile
            .moderation_storage()
            .waiting_room_add_participant(self.room_id.room_id(), self.id)
            .await;

        self.volatile.room_locking().unlock_room(guard).await?;
        let added_to_waiting_room = res?;

        // Check the participant id has not been added to the waiting room before. That woul be a
        // duplicate which cannot be allowed. Since this should never happen just error and exit.
        if !self.resuming && !added_to_waiting_room {
            whatever!("participant-id is already taken inside waiting-room set");
        }

        self.state = RunnerState::Waiting {
            accepted: false,
            control_data,
        };

        self.ws
            .send(Message::Text(
                serde_json::to_string(&NamespacedEvent {
                    module: opentalk_types_signaling_moderation::MODULE_ID,
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
                module: opentalk_types_signaling_moderation::MODULE_ID,
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
        // Clear the left_at timestamp to indicate that the participant is in the real room (not just the waiting room).
        let control_data = ControlState {
            left_at: None,
            ..control_data
        };
        self.volatile
            .control_storage()
            .remove_local_attribute(self.id, self.room_id, LEFT_AT)
            .await?;
        self.volatile
            .control_storage()
            .set_local_attribute(self.id, self.room_id, JOINED_AT, timestamp)
            .await?;

        // If we haven't joined the waiting room yet, fetch, set and enforce the tariff for the room.
        // When in waiting-room this logic was already executed in `join_waiting_room`.
        let (guard, tariff) = if !joining_from_waiting_room {
            let creator_id = self.room.created_by;

            let mut tariff = Tariff::get_by_user_id(&mut self.db.get_conn().await?, &creator_id)
                .await
                .whatever_context::<_, RunnerError>("Failed to get user")?;

            let guard = self.volatile.room_locking().lock_room(self.room_id).await?;

            match self.enforce_tariff(tariff.clone()).await {
                Ok(ControlFlow::Continue(enforced_tariff)) => {
                    tariff = enforced_tariff;
                }
                Ok(ControlFlow::Break(reason)) => {
                    self.volatile.room_locking().unlock_room(guard).await?;

                    self.ws_send_control(Timestamp::now(), ControlEvent::JoinBlocked(reason))
                        .await;

                    return Ok(());
                }
                Err(e) => {
                    self.volatile.room_locking().unlock_room(guard).await?;

                    return Err(e);
                }
            };

            (guard, tariff)
        } else {
            let tariff = self
                .volatile
                .control_storage()
                .get_tariff(self.room.id)
                .await?;
            (
                self.volatile.room_locking().lock_room(self.room_id).await?,
                tariff,
            )
        };

        let res = self.join_room_locked().await;

        let unlock_res = self.volatile.room_locking().unlock_room(guard).await;

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
        let event = self
            .volatile
            .control_storage()
            .try_init_event(self.room.id, event)
            .await?;

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

        let closes_at = self
            .volatile
            .control_storage()
            .get_room_closes_at(self.room_id)
            .await?;

        let settings = self.settings_provider.get();

        let mut module_features = BTreeMap::<ModuleId, BTreeSet<FeatureId>>::new();
        self.modules
            .get_module_features()
            .iter()
            .for_each(|(k, v)| {
                module_features.insert(k.clone(), v.clone());
            });

        let tariff_resource = tariff
            .to_tariff_resource(settings.defaults.disabled_features.clone(), module_features)
            .into();

        let mut conn = self.db.get_conn().await?;
        let event_info = match event.as_ref() {
            Some(event) => {
                let call_in_tel = settings.call_in.as_ref().map(|call_in| call_in.tel.clone());
                Some(
                    build_event_info(
                        &mut conn,
                        call_in_tel,
                        self.room.id,
                        self.room.e2e_encryption,
                        event,
                        &tariff,
                    )
                    .await?,
                )
            }
            _ => None,
        };

        let room_info = self
            .build_room_info(&mut conn, &settings.avatar.libravatar_url)
            .await?;

        self.ws_send_control(
            timestamp,
            ControlEvent::JoinSuccess(Box::new(JoinSuccess {
                id: self.id,
                display_name: control_data.display_name.clone(),
                avatar_url: control_data.avatar_url.clone(),
                role: self.role,
                closes_at,
                tariff: tariff_resource,
                module_data,
                participants,
                event_info,
                room_info,
                is_room_owner: self.participant.user_id() == Some(self.room.created_by),
            })),
        )
        .await;

        self.state = RunnerState::Joined;

        self.exchange_publish_control(timestamp, None, exchange::Message::Joined(self.id));

        self.handle_module_requested_actions(timestamp, actions)
            .await;

        Ok(())
    }

    async fn join_room_locked(&mut self) -> Result<BTreeSet<ParticipantId>> {
        let participant_set_exists = self
            .volatile
            .control_storage()
            .participant_set_exists(self.room_id)
            .await?;

        if !participant_set_exists {
            self.set_room_time_limit().await?;

            if self.room_id.breakout_room_id().is_some() {
                self.metrics.increment_created_breakout_rooms_count();
            } else {
                self.metrics
                    .record_room_creation_metrics(self.room_id.room_id());
            }

            self.volatile
                .control_storage()
                .set_room_alive(self.room_id.room_id())
                .await?;
        }

        self.activate_room_time_limit().await?;

        let participants = self
            .volatile
            .control_storage()
            .get_all_participants(self.room_id)
            .await?;

        let added = self
            .volatile
            .control_storage()
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
        let tariff = self
            .volatile
            .control_storage()
            .get_tariff(self.room.id)
            .await?;

        let remaining_seconds = tariff
            .quota(&QuotaType::RoomTimeLimitSecs)
            .map(|time_limit| i64::try_from(time_limit).unwrap_or(i64::MAX));

        if let Some(remaining_seconds) = remaining_seconds {
            let closes_at =
                Timestamp::now().checked_add_signed(chrono::Duration::seconds(remaining_seconds));

            if let Some(closes_at) = closes_at {
                self.volatile
                    .control_storage()
                    .set_room_closes_at(self.room_id, closes_at.into())
                    .await?;
            } else {
                log::error!("DateTime overflow for closes_at");
            }
        }

        Ok(())
    }

    async fn activate_room_time_limit(&mut self) -> Result<()> {
        let closes_at = self
            .volatile
            .control_storage()
            .get_room_closes_at(self.room_id)
            .await?;

        if let Some(closes_at) = closes_at {
            let remaining_seconds = (*closes_at - *Timestamp::now()).num_seconds();
            let future = sleep(Duration::from_secs(remaining_seconds.max(0) as u64));
            self.time_limit_future = Box::pin(future);
        }

        Ok(())
    }

    async fn set_control_attributes(
        &mut self,
        timestamp: Timestamp,
        display_name: &DisplayName,
        avatar_url: Option<&str>,
    ) -> Result<()> {
        let mut actions = AttributeActions::new(self.room_id, self.id);

        match &self.participant {
            Participant::User(user) => {
                actions
                    .set_local(KIND, ParticipationKind::User)
                    .set_local(
                        AVATAR_URL,
                        avatar_url.expect("user must have avatar_url set"),
                    )
                    .set_local(USER_ID, user.id)
                    .set_global(IS_ROOM_OWNER, user.id == self.room.created_by);
            }
            Participant::Guest => {
                actions.set_local(KIND, ParticipationKind::Guest);
            }
            Participant::Sip => {
                actions.set_local(KIND, ParticipationKind::Sip);
            }
            Participant::Recorder => {
                actions.set_local(KIND, ParticipationKind::Recorder);
            }
        }

        self.volatile
            .control_storage()
            .bulk_attribute_actions::<()>(
                actions
                    .set_global(ROLE, self.role)
                    .set_global(IS_PRESENT, true)
                    .set_global(BREAKOUT_ROOM, self.room_id.breakout_room_id())
                    .set_local(HAND_IS_UP, false)
                    .set_local(HAND_UPDATED_AT, timestamp)
                    .set_global(DISPLAY_NAME, display_name)
                    .set_local(JOINED_AT, timestamp),
            )
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
    ) -> Result<Option<opentalk_types_signaling::Participant>> {
        let mut participant = opentalk_types_signaling::Participant {
            id,
            module_data: Default::default(),
        };

        let control_data =
            ControlState::from_storage(self.volatile.control_storage(), self.room_id, id).await?;

        // Do not build participants for invisible services
        if control_data.participation_kind.visibility().is_hidden() {
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

        if namespaced.module == MODULE_ID {
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
                    &namespaced.module,
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
                if self.id == id && reason == LeaveReason::SentToWaitingRoom {
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
                        self.volatile
                            .moderation_storage()
                            .set_skip_waiting_room_with_expiry(self.id, true)
                            .await?;

                        self.ws
                            .send(Message::Text(
                                serde_json::to_string(&NamespacedEvent {
                                    module: opentalk_types_signaling_moderation::MODULE_ID,
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

                self.volatile
                    .control_storage()
                    .set_global_attribute(self.id, self.room_id.room_id(), ROLE, new_role)
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

                self.ws_send_control(
                    timestamp,
                    ControlEvent::RoleUpdated(RoleUpdated { new_role }),
                )
                .await;

                self.exchange_publish_control(timestamp, None, exchange::Message::Update(self.id));
            }
            exchange::Message::ResetRaisedHands { issued_by } => {
                let raised: Option<bool> = self
                    .volatile
                    .control_storage()
                    .get_local_attribute(self.id, self.room_id, HAND_IS_UP)
                    .await?;
                if matches!(raised, Some(true)) {
                    self.handle_raise_hand_change(timestamp, false).await?;

                    self.ws
                        .send(Message::Text(
                            serde_json::to_string(&NamespacedEvent {
                                module: opentalk_types_signaling_moderation::MODULE_ID,
                                timestamp,
                                payload: ModerationEvent::RaisedHandResetByModerator(
                                    RaisedHandResetByModerator { issued_by },
                                ),
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
                            module: opentalk_types_signaling_moderation::MODULE_ID,
                            timestamp,
                            payload: ModerationEvent::RaiseHandsEnabled(RaiseHandsEnabled {
                                issued_by,
                            }),
                        })
                        .whatever_context::<_, RunnerError>("Failed to send ws message")?
                        .into(),
                    ))
                    .await;
            }
            exchange::Message::DisableRaiseHands { issued_by } => {
                let raised: Option<bool> = self
                    .volatile
                    .control_storage()
                    .get_local_attribute(self.id, self.room_id, HAND_IS_UP)
                    .await?;
                if matches!(raised, Some(true)) {
                    self.handle_raise_hand_change(timestamp, false).await?;
                }

                self.ws
                    .send(Message::Text(
                        serde_json::to_string(&NamespacedEvent {
                            module: opentalk_types_signaling_moderation::MODULE_ID,
                            timestamp,
                            payload: ModerationEvent::RaiseHandsDisabled(RaiseHandsDisabled {
                                issued_by,
                            }),
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

        self.volatile
            .control_storage()
            .set_local_attribute(self.id, self.room_id, LEFT_AT, Timestamp::now())
            .await?;

        let actions = self
            .handle_module_broadcast_event(timestamp, DynBroadcastEvent::Leaving, false)
            .await;

        self.handle_module_requested_actions(timestamp, actions)
            .await;

        // resuming ensures that we can reuse the same participant ID
        self.resuming = true;
        self.volatile
            .moderation_storage()
            .waiting_room_add_participant(self.room_id.room_id(), self.id)
            .await?;

        let control_data =
            ControlState::from_storage(self.volatile.control_storage(), self.room_id, self.id)
                .await?;

        self.state = RunnerState::Waiting {
            accepted: false,
            control_data,
        };

        self.ws
            .send(Message::Text(
                serde_json::to_string(&NamespacedEvent {
                    module: opentalk_types_signaling_moderation::MODULE_ID,
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
                module: opentalk_types_signaling_moderation::MODULE_ID,
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
            module: MODULE_ID,
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
        module: &ModuleId,
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
            volatile: &mut self.volatile,
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
            volatile: &mut self.volatile,
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

        if let Some((close_code, reason)) = exit {
            self.leave_reason = reason;
            self.ws.close(close_code).await;
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
                    module: MODULE_ID,
                    timestamp,
                    payload,
                })
                .expect("Failed to convert namespaced to json")
                .into(),
            ))
            .await;
    }

    async fn notify_left(&mut self, id: ParticipantId, reason: LeaveReason, timestamp: Timestamp) {
        let actions: ModuleRequestedActions = self
            .handle_module_broadcast_event(timestamp, DynBroadcastEvent::ParticipantLeft(id), false)
            .await;

        if self.id != id {
            self.ws_send_control(
                timestamp,
                ControlEvent::Left(Left {
                    id: AssociatedParticipant { id },
                    reason,
                }),
            )
            .await;
        }

        self.handle_module_requested_actions(timestamp, actions)
            .await;
    }

    async fn username_or(&self, join_display_name: Option<DisplayName>) -> DisplayName {
        let join_display_name = join_display_name.unwrap_or_default();

        match &self.participant {
            Participant::User(user) => {
                // Enforce the auto-generated display name if display name editing is prohibited
                if self
                    .settings_provider
                    .get()
                    .endpoints
                    .disallow_custom_display_name
                {
                    user.display_name.clone()
                } else {
                    join_display_name.clone()
                }
            }
            Participant::Guest => join_display_name,
            Participant::Recorder => join_display_name,
            Participant::Sip => {
                if let Some(call_in) = self.settings_provider.get().call_in.as_ref() {
                    call_in::display_name(&self.db, call_in, self.room.tenant_id, join_display_name)
                        .await
                } else {
                    join_display_name
                }
            }
        }
    }

    async fn build_room_info(
        &mut self,
        conn: &mut DbConnection,
        libravatar_url: &str,
    ) -> Result<RoomInfo> {
        if let Some(creator_info) = self
            .volatile
            .control_storage()
            .get_creator(self.room.id)
            .await?
        {
            return Ok(RoomInfo {
                id: self.room.id,
                password: self.room.password.clone(),
                created_by: creator_info,
            });
        }

        let creator = User::get(conn, self.room.created_by).await?;

        let creator_info = UserInfo {
            title: creator.title,
            firstname: creator.firstname,
            lastname: creator.lastname,
            display_name: creator.display_name,
            avatar_url: creator
                .avatar_url
                .unwrap_or_else(|| email_to_libravatar_url(libravatar_url, &creator.email)),
        };

        let creator_info = self
            .volatile
            .control_storage()
            .try_init_creator(self.room.id, creator_info)
            .await?;

        Ok(RoomInfo {
            id: self.room.id,
            password: self.room.password.clone(),
            created_by: creator_info,
        })
    }

    async fn avatar_url(&self) -> Option<String> {
        match &self.participant {
            Participant::User(user) => Some(user.avatar_url.clone().unwrap_or_else(|| {
                let settings = self.settings_provider.get();
                format!(
                    "{}{:x}",
                    settings.avatar.libravatar_url,
                    md5::compute(&user.email)
                )
            })),
            Participant::Guest | Participant::Recorder | Participant::Sip => None,
        }
    }

    /// Check the given exchange message and determine if a participant joined
    ///
    /// Returns an appropriate [`JoinEvent`] or [`None`] when no one joined.
    fn has_participant_joined(&self, msg: ByteString) -> Option<JoinEvent> {
        let namespaced = match serde_json::from_str::<NamespacedEvent<Value>>(&msg) {
            Ok(namespaced) => namespaced,
            Err(e) => {
                log::error!(
                    "Failed to read incoming exchange message, {}",
                    Report::from_error(e)
                );
                return None;
            }
        };

        if namespaced.module == control::MODULE_ID {
            if let Ok(exchange::Message::Joined(_)) =
                serde_json::from_value::<exchange::Message>(namespaced.payload)
            {
                return Some(JoinEvent::Room(self.room_id));
            }
        } else if namespaced.module == opentalk_types_signaling_moderation::MODULE_ID {
            if let Ok(moderation::exchange::Message::JoinedWaitingRoom(_)) =
                serde_json::from_value::<moderation::exchange::Message>(namespaced.payload)
            {
                return Some(JoinEvent::WaitingRoom);
            }
        } else if namespaced.module == opentalk_types_signaling_breakout::MODULE_ID {
            if let Ok(breakout::exchange::Message::Joined(participant_joined_other_room)) =
                serde_json::from_value::<breakout::exchange::Message>(namespaced.payload)
            {
                return Some(JoinEvent::Room(SignalingRoomId::new(
                    self.room_id.room_id(),
                    participant_joined_other_room.breakout_room,
                )));
            }
        }

        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum JoinEvent {
    /// A participant joined the waiting room
    WaitingRoom,
    /// A participant joined the either the main or breakout room
    Room(SignalingRoomId),
}

/// Remove all room and control module related data from redis for the given 'local' room/breakout-room. Does not
/// touch any keys that contain 'global' data that is used across all 'sub'-rooms (main & breakout rooms).
async fn cleanup_redis_keys_for_signaling_room(
    storage: &mut VolatileStorage,
    room_id: SignalingRoomId,
) -> Result<()> {
    storage
        .control_storage()
        .remove_room_closes_at(room_id)
        .await?;
    storage
        .control_storage()
        .remove_participant_set(room_id)
        .await?;

    for attribute in [
        JOINED_AT,
        LEFT_AT,
        HAND_IS_UP,
        HAND_UPDATED_AT,
        KIND,
        USER_ID,
        AVATAR_URL,
    ] {
        storage
            .control_storage()
            .remove_attribute_key(
                LocalRoomAttributeId {
                    room: room_id,
                    attribute,
                }
                .into(),
            )
            .await?;
    }

    if room_id.breakout_room_id().is_none() {
        for attribute in [ROLE, DISPLAY_NAME, IS_PRESENT, IS_ROOM_OWNER, BREAKOUT_ROOM] {
            storage
                .control_storage()
                .remove_attribute_key(
                    GlobalRoomAttributeId {
                        room: room_id.room_id(),
                        attribute,
                    }
                    .into(),
                )
                .await?;
        }
    }

    Ok(())
}

#[must_use]
struct ModuleRequestedActions {
    ws_messages: Vec<Message>,
    exchange_publish: Vec<ExchangePublish>,
    invalidate_data: bool,
    exit: Option<(CloseCode, LeaveReason)>,
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
