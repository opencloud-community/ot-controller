// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use chrono::{Duration, Utc};
use exchange::GenerateUrl;
use opentalk_database::Db;
use opentalk_etherpad_client::EtherpadClient;
use opentalk_signaling_core::{
    assets::{save_asset, AssetError, NewAssetFileName},
    control::{
        self,
        storage::{ControlStorage as _, ControlStorageParticipantAttributes as _, DISPLAY_NAME},
    },
    DestroyContext, Event, InitContext, ModuleContext, ObjectStorage, RedisConnection,
    SignalingModule, SignalingModuleError, SignalingModuleInitData, SignalingRoomId,
    VolatileStorageBackend,
};
use opentalk_types::{
    core::{FileExtension, ParticipantId},
    signaling::{
        protocol::{
            command::{ParticipantSelection, ProtocolCommand},
            event::{AccessUrl, Error, PdfAsset, ProtocolEvent},
            peer_state::ProtocolPeerState,
            NAMESPACE,
        },
        Role,
    },
};
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use snafu::{whatever, OptionExt, Report};
use storage::InitState;

use crate::storage::ProtocolStorage;

pub mod exchange;
pub mod storage;

const PAD_NAME: &str = "protocol";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToRedisArgs, FromRedisValue)]
#[to_redis_args(serde)]
#[from_redis_value(serde)]
struct SessionInfo {
    author_id: String,
    group_id: String,
    session_id: String,
    readonly: bool,
}

pub struct Protocol {
    etherpad: EtherpadClient,
    participant_id: ParticipantId,
    room_id: SignalingRoomId,
    db: Arc<Db>,
    storage: Arc<ObjectStorage>,
}

#[derive(Clone)]
pub struct VolatileWrapper {
    storage: VolatileStorageBackend,
}

impl From<VolatileStorageBackend> for VolatileWrapper {
    fn from(storage: VolatileStorageBackend) -> Self {
        Self { storage }
    }
}

impl VolatileWrapper {
    fn storage_ref(&self) -> &dyn storage::ProtocolStorage {
        if self.storage.is_left() {
            self.storage.as_ref().left().unwrap()
        } else {
            self.storage.as_ref().right().unwrap()
        }
    }

    fn storage_mut(&mut self) -> &mut dyn storage::ProtocolStorage {
        if self.storage.is_left() {
            self.storage.as_mut().left().unwrap()
        } else {
            self.storage.as_mut().right().unwrap()
        }
    }
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for Protocol {
    const NAMESPACE: &'static str = NAMESPACE;
    type Params = opentalk_controller_settings::Etherpad;
    type Incoming = ProtocolCommand;
    type Outgoing = ProtocolEvent;
    type ExchangeMessage = exchange::Event;
    type ExtEvent = ();
    type FrontendData = ();
    type PeerFrontendData = ProtocolPeerState;

    type Volatile = VolatileWrapper;

    async fn init(
        ctx: InitContext<'_, Self>,
        params: &Self::Params,
        _protocol: &'static str,
    ) -> Result<Option<Self>, SignalingModuleError> {
        let etherpad = EtherpadClient::new(params.url.clone(), params.api_key.clone());

        Ok(Some(Self {
            etherpad,
            participant_id: ctx.participant_id(),
            room_id: ctx.room_id(),
            db: ctx.db().clone(),
            storage: ctx.storage().clone(),
        }))
    }

    async fn on_event(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        event: Event<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        match event {
            // Create a readonly session for every joining participant when the protocol module is already initialized
            //
            // Send the current access state of every participant when the joining participant is a moderator
            Event::Joined {
                control_data: _,
                frontend_data: _,
                participants,
            } => {
                let state = ctx.redis_conn().init_get(self.room_id).await?;

                if matches!(state, Some(state) if state == InitState::Initialized) {
                    let read_url = self.generate_url(&mut ctx, true).await?;

                    ctx.ws_send(ProtocolEvent::ReadUrl(AccessUrl { url: read_url }));

                    for (participant_id, access) in participants {
                        let session_info = ctx
                            .redis_conn()
                            .session_get(self.room_id, *participant_id)
                            .await?;

                        *access = session_info.map(|session_info| ProtocolPeerState {
                            readonly: session_info.readonly,
                        });
                    }
                }
            }
            Event::Leaving => {
                if let Some(session_info) = ctx
                    .redis_conn()
                    .session_delete(self.room_id, self.participant_id)
                    .await?
                {
                    self.etherpad
                        .delete_session(&session_info.session_id)
                        .await?
                }
            }
            Event::WsMessage(msg) => {
                self.on_ws_message(&mut ctx, msg).await?;
            }
            Event::Exchange(event) => {
                self.on_exchange_event(&mut ctx, event).await?;
            }
            Event::ParticipantUpdated(participant_id, peer_frontend_data)
            | Event::ParticipantJoined(participant_id, peer_frontend_data) => {
                let session_info = ctx
                    .redis_conn()
                    .session_get(self.room_id, participant_id)
                    .await?;

                *peer_frontend_data = session_info.map(|session_info| ProtocolPeerState {
                    readonly: session_info.readonly,
                });
            }
            _ => (),
        }

        Ok(())
    }

    async fn on_destroy(self, mut ctx: DestroyContext<'_>) {
        if ctx.destroy_room() {
            if let Err(e) = self.cleanup_etherpad(ctx.redis_conn()).await {
                log::error!(
                    "Failed to cleanup etherpad for room {} in redis: {}",
                    self.room_id,
                    e
                );
            }

            if let Err(e) = ctx.redis_conn().cleanup(self.room_id).await {
                log::error!(
                    "Failed to cleanup protocol keys for room {} in redis: {}",
                    self.room_id,
                    e
                );
            }
        }
    }

    async fn build_params(
        init: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
        let etherpad = init.shared_settings.load_full().etherpad.clone();

        match etherpad {
            Some(etherpad) => Ok(Some(etherpad)),
            None => {
                log::warn!(
                    "Skipping the Protocol module as no etherpad is specified in the config"
                );
                Ok(None)
            }
        }
    }
}

impl Protocol {
    async fn on_ws_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        msg: ProtocolCommand,
    ) -> Result<(), SignalingModuleError> {
        match msg {
            ProtocolCommand::SelectWriter(selection) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::InsufficientPermissions);

                    return Ok(());
                }

                if !self.verify_selection(ctx.redis_conn(), &selection).await? {
                    ctx.ws_send(Error::InvalidParticipantSelection);
                }

                let targets = selection.participant_ids;

                // TODO: disallow selecting the same participant twice

                let redis_conn = ctx.redis_conn();

                let init_state = redis_conn.try_start_init(self.room_id).await?;

                let first_init = match init_state {
                    Some(state) => {
                        match state {
                            InitState::Initializing => {
                                // Some other instance is currently initializing the etherpad
                                ctx.ws_send(Error::CurrentlyInitializing);
                                return Ok(());
                            }
                            InitState::Initialized => false,
                        }
                    }
                    None => {
                        // No init state was set before -> Initialize the etherpad in this module instance
                        if let Err(e) = self.init_etherpad(redis_conn).await {
                            log::error!(
                                "Failed to init etherpad for room {}, {}",
                                self.room_id,
                                Report::from_error(e)
                            );

                            redis_conn.init_delete(self.room_id).await?;

                            ctx.ws_send(Error::FailedInitialization);

                            return Ok(());
                        }

                        true
                    }
                };

                if first_init {
                    // all participants get access on first initialization
                    ctx.exchange_publish(
                        control::exchange::current_room_all_participants(self.room_id),
                        exchange::Event::GenerateUrl(GenerateUrl { writers: targets }),
                    );
                } else {
                    // calls after the first init only reach the targeted participants
                    for participant_id in targets {
                        ctx.exchange_publish(
                            control::exchange::current_room_by_participant_id(
                                self.room_id,
                                participant_id,
                            ),
                            exchange::Event::GenerateUrl(GenerateUrl {
                                writers: vec![participant_id],
                            }),
                        );
                    }
                }
            }
            ProtocolCommand::DeselectWriter(selection) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::InsufficientPermissions);

                    return Ok(());
                }

                match ctx.redis_conn().init_get(self.room_id).await? {
                    Some(state) => match state {
                        InitState::Initializing => {
                            ctx.ws_send(Error::CurrentlyInitializing);

                            return Ok(());
                        }
                        InitState::Initialized => (),
                    },
                    None => {
                        ctx.ws_send(Error::NotInitialized);

                        return Ok(());
                    }
                }

                if !self.verify_selection(ctx.redis_conn(), &selection).await? {
                    ctx.ws_send(Error::InvalidParticipantSelection);

                    return Ok(());
                }

                for participant_id in selection.participant_ids {
                    // check if its actually a writer
                    let session_info = ctx
                        .redis_conn()
                        .session_get(self.room_id, participant_id)
                        .await?;

                    let session_info = if let Some(session_info) = session_info {
                        session_info
                    } else {
                        continue;
                    };

                    // check if session is readonly already
                    if session_info.readonly {
                        continue;
                    }

                    // notify participant to recreate readonly sessions
                    ctx.exchange_publish(
                        control::exchange::current_room_by_participant_id(
                            self.room_id,
                            participant_id,
                        ),
                        exchange::Event::GenerateUrl(GenerateUrl { writers: vec![] }),
                    );
                }
            }
            ProtocolCommand::GeneratePdf => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::InsufficientPermissions);
                    return Ok(());
                }

                if !matches!(
                    ctx.redis_conn().init_get(self.room_id).await?,
                    Some(InitState::Initialized)
                ) {
                    ctx.ws_send(Error::NotInitialized);
                    return Ok(());
                }

                let session_info = ctx
                    .redis_conn()
                    .session_get(self.room_id, self.participant_id)
                    .await?;
                if let Some(session_info) = session_info {
                    let group_id = ctx.redis_conn().group_get(self.room_id).await?.unwrap();

                    let pad_id = format!("{group_id}${PAD_NAME}");

                    let data = self
                        .etherpad
                        .download_pdf(&session_info.session_id, &pad_id)
                        .await?;

                    let kind = "meetingnotes_pdf"
                        .parse()
                        .expect("Must be parseable as AssetFileKind");

                    let filename =
                        NewAssetFileName::new(kind, ctx.timestamp(), FileExtension::pdf());

                    let (asset_id, filename) = match save_asset(
                        &self.storage,
                        self.db.clone(),
                        self.room_id.room_id(),
                        Some(Self::NAMESPACE),
                        filename,
                        data,
                    )
                    .await
                    {
                        Ok(asset_id) => asset_id,
                        Err(AssetError::AssetStorageExceeded) => {
                            ctx.ws_send(Error::StorageExceeded);
                            return Ok(());
                        }

                        Err(e) => {
                            let message = format!(
                                "Failed to save meetingnotes asset: {}",
                                Report::from_error(e)
                            );
                            log::error!("{message}");
                            whatever!("{message}");
                        }
                    };

                    ctx.exchange_publish(
                        control::exchange::current_room_all_participants(self.room_id),
                        exchange::Event::PdfAsset(PdfAsset { filename, asset_id }),
                    );
                } else {
                    ctx.ws_send(Error::NotInitialized);
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    async fn on_exchange_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: exchange::Event,
    ) -> Result<(), SignalingModuleError> {
        match event {
            exchange::Event::GenerateUrl(GenerateUrl { writers }) => {
                if writers.contains(&self.participant_id) {
                    let write_url = self.generate_url(ctx, false).await?;

                    ctx.ws_send(ProtocolEvent::WriteUrl(AccessUrl { url: write_url }));
                } else {
                    let read_url = self.generate_url(ctx, true).await?;

                    ctx.ws_send(ProtocolEvent::ReadUrl(AccessUrl { url: read_url }));
                }

                ctx.invalidate_data();
            }
            exchange::Event::PdfAsset(pdf_asset) => ctx.ws_send(pdf_asset),
        }

        Ok(())
    }

    /// Initializes the etherpad-group and -pad for this room
    async fn init_etherpad(
        &self,
        redis_conn: &mut RedisConnection,
    ) -> Result<(), SignalingModuleError> {
        let group_id = self
            .etherpad
            .create_group_for(self.room_id.to_string())
            .await?;

        self.etherpad
            .create_group_pad(&group_id, PAD_NAME, None)
            .await?;

        redis_conn.group_set(self.room_id, &group_id).await?;

        // flag this room as initialized
        redis_conn.set_initialized(self.room_id).await?;

        Ok(())
    }

    /// Creates a new etherpad author for the participant
    ///
    /// Returns the generated author id
    async fn create_author(
        &self,
        redis_conn: &mut RedisConnection,
    ) -> Result<String, SignalingModuleError> {
        let display_name: String = redis_conn
            .get_attribute(self.room_id, self.participant_id, DISPLAY_NAME)
            .await?;

        let author_id = self
            .etherpad
            .create_author_if_not_exits_for(&display_name, &self.participant_id.to_string())
            .await?;

        Ok(author_id)
    }

    /// Creates a new etherpad session
    ///
    /// Returns the generated session id
    async fn create_session(
        &self,
        group_id: &str,
        author_id: &str,
        expire_duration: Duration,
        readonly: bool,
    ) -> Result<String, SignalingModuleError> {
        let expires = Utc::now()
            .checked_add_signed(expire_duration)
            .whatever_context::<&str, SignalingModuleError>("DateTime overflow")?
            .timestamp();

        let session_id = if readonly {
            self.etherpad
                .create_read_session(group_id, author_id, expires)
                .await?
        } else {
            self.etherpad
                .create_session(group_id, author_id, expires)
                .await?
        };

        Ok(session_id)
    }

    /// Creates a new author & session for a the participant
    ///
    /// Returns the `[SessionInfo]`
    async fn prepare_and_create_user_session(
        &mut self,
        redis_conn: &mut RedisConnection,
        readonly: bool,
    ) -> Result<SessionInfo, SignalingModuleError> {
        let author_id = self.create_author(redis_conn).await?;

        let group_id = redis_conn
            .group_get(self.room_id)
            .await?
            .whatever_context::<&str, SignalingModuleError>(
                "Missing group for room while preparing a new session",
            )?;

        // Currently there is no proper session refresh in etherpad. Due to the the difficulty of setting new sessions
        // on the client across domains, we set the expire duration to 14 days and hope for the best.
        // Session refresh is merged and will be available in the next release: https://github.com/ether/etherpad-lite/pull/5361
        // TODO: use proper session refresh from etherpad once it's released
        let session_id = self
            .create_session(&group_id, &author_id, Duration::days(14), readonly)
            .await?;

        let session_info = SessionInfo {
            author_id,
            group_id,
            session_id,
            readonly,
        };

        redis_conn
            .session_set(self.room_id, self.participant_id, &session_info)
            .await?;

        Ok(session_info)
    }

    /// Generates the auth-session url
    ///
    /// Creates a new user session which has ether write or read access, depending on the `readonly` flag.
    /// Any existing etherpad session for the participant will be removed and replaced.
    async fn generate_url(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        readonly: bool,
    ) -> Result<String, SignalingModuleError> {
        let redis_conn = ctx.redis_conn();

        // remove existing sessions from redis
        if let Some(session_info) = redis_conn
            .session_delete(self.room_id, self.participant_id)
            .await?
        {
            // If any exists, remove the participants session from the etherpad instance
            self.etherpad
                .delete_session(&session_info.session_id)
                .await?;
        }

        let session_info = self
            .prepare_and_create_user_session(redis_conn, readonly)
            .await?;

        let url = self.etherpad.auth_session_url(
            &session_info.session_id,
            PAD_NAME,
            Some(&session_info.group_id),
        )?;

        // Notify other participants about the access change

        Ok(url.to_string())
    }

    /// Checks if all provided targets in a [`ParticipantSelection`] exist in this room
    ///
    /// Returns true when all targets are recognized
    async fn verify_selection(
        &self,
        redis_conn: &mut RedisConnection,
        selection: &ParticipantSelection,
    ) -> Result<bool, SignalingModuleError> {
        let room_participants = redis_conn.get_all_participants(self.room_id).await?;

        Ok(selection
            .participant_ids
            .iter()
            .all(|target| room_participants.contains(target)))
    }

    /// Removes the room related pad and group from etherpad
    async fn cleanup_etherpad(
        &self,
        redis_conn: &mut RedisConnection,
    ) -> Result<(), SignalingModuleError> {
        let init_state = redis_conn.init_get(self.room_id).await?;

        if init_state.is_none() {
            // Nothing to cleanup
            return Ok(());
        }

        let group_id = redis_conn.group_get(self.room_id).await?.unwrap();

        let pad_id = format!("{group_id}${PAD_NAME}");

        self.etherpad.delete_pad(&pad_id).await?;

        // invalidate all sessions by deleting the group
        self.etherpad.delete_group(&group_id).await?;

        Ok(())
    }
}
