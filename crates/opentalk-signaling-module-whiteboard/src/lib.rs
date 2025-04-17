// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use client::SpacedeckClient;
use either::Either;
use futures::stream::once;
use opentalk_database::Db;
use opentalk_signaling_core::{
    assets::{save_asset, AssetError, NewAssetFileName},
    control, ChunkFormat, CleanupScope, DestroyContext, Event, InitContext, ModuleContext,
    ObjectStorage, SignalingModule, SignalingModuleError, SignalingModuleInitData, SignalingRoomId,
    VolatileStorage,
};
use opentalk_types_common::{
    assets::{asset_file_kind, AssetFileKind, FileExtension},
    modules::ModuleId,
    time::Timestamp,
};
use opentalk_types_signaling::Role;
use opentalk_types_signaling_whiteboard::{
    command::WhiteboardCommand,
    event::{AccessUrl, Error, PdfAsset, WhiteboardEvent},
    state::WhiteboardState,
    MODULE_ID,
};
use snafu::{whatever, Report};
use storage::{InitState, SpaceInfo, WhiteboardStorage};
use url::Url;

mod client;
mod exchange;
mod storage;

pub struct Whiteboard {
    room_id: SignalingRoomId,
    client: SpacedeckClient,
    db: Arc<Db>,
    storage: Arc<ObjectStorage>,
}

impl From<InitState> for WhiteboardState {
    fn from(init_state: InitState) -> Self {
        match init_state {
            InitState::Initializing => Self::Initializing,
            InitState::Initialized(info) => Self::Initialized(info.url),
        }
    }
}

pub struct GetPdfEvent {
    url_result: Result<Url, SignalingModuleError>,
    timestamp: Timestamp,
}

trait WhiteboardStorageProvider {
    fn storage(&mut self) -> &mut dyn WhiteboardStorage;
}

impl WhiteboardStorageProvider for VolatileStorage {
    fn storage(&mut self) -> &mut dyn WhiteboardStorage {
        match self.as_mut() {
            Either::Left(v) => v,
            Either::Right(v) => v,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for Whiteboard {
    const NAMESPACE: ModuleId = MODULE_ID;

    type Params = opentalk_controller_settings::settings_file::Spacedeck;

    type Incoming = WhiteboardCommand;

    type Outgoing = WhiteboardEvent;

    type ExchangeMessage = exchange::Event;

    type ExtEvent = GetPdfEvent;

    type FrontendData = WhiteboardState;

    type PeerFrontendData = ();

    async fn init(
        ctx: InitContext<'_, Self>,
        params: &Self::Params,
        _protocol: &'static str,
    ) -> Result<Option<Self>, SignalingModuleError> {
        let client = SpacedeckClient::new(params.url.clone(), params.api_key.clone());

        Ok(Some(Self {
            room_id: ctx.room_id(),
            client,
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
            Event::Joined {
                control_data: _,
                frontend_data,
                participants: _,
            } => {
                let data = match ctx.volatile.storage().get_init_state(self.room_id).await? {
                    Some(state) => state.into(),
                    None => WhiteboardState::NotInitialized,
                };

                *frontend_data = Some(data);

                Ok(())
            }
            Event::Exchange(event) => {
                match event {
                    exchange::Event::Initialized => {
                        if let Some(InitState::Initialized(space_info)) =
                            ctx.volatile.storage().get_init_state(self.room_id).await?
                        {
                            ctx.ws_send(AccessUrl {
                                url: space_info.url,
                            });
                        } else {
                            log::error!("Whiteboard module received `Initialized` but spacedeck was not initialized");
                        }
                    }
                    exchange::Event::PdfAsset(pdf_asset) => {
                        ctx.ws_send(pdf_asset);
                    }
                }
                Ok(())
            }

            Event::WsMessage(message) => {
                match message {
                    WhiteboardCommand::Initialize => {
                        if ctx.role() != Role::Moderator {
                            ctx.ws_send(Error::InsufficientPermissions);
                            return Ok(());
                        }

                        if let Err(err) = self.create_space(&mut ctx).await {
                            log::error!(
                                "Failed to initialize whiteboard for room '{}': {}",
                                self.room_id,
                                err
                            );

                            self.cleanup_room(ctx.volatile.storage(), self.room_id)
                                .await;

                            ctx.ws_send(Error::InitializationFailed);
                        }
                    }

                    WhiteboardCommand::GeneratePdf => {
                        if ctx.role() != Role::Moderator {
                            ctx.ws_send(Error::InsufficientPermissions);
                            return Ok(());
                        }

                        if let Some(storage::InitState::Initialized(info)) =
                            ctx.volatile.storage().get_init_state(self.room_id).await?
                        {
                            let client = self.client.clone();
                            let timestamp = ctx.timestamp();

                            ctx.add_event_stream(once(async move {
                                GetPdfEvent {
                                    url_result: client.get_pdf(&info.id).await,
                                    timestamp,
                                }
                            }));
                        }
                    }
                }
                Ok(())
            }
            Event::Ext(GetPdfEvent {
                url_result,
                timestamp,
            }) => {
                let url = url_result?;

                let data = self.client.download_pdf(url.clone()).await?;

                const ASSET_FILE_KIND: AssetFileKind = asset_file_kind!("whiteboard_pdf");
                let filename =
                    NewAssetFileName::new(ASSET_FILE_KIND, timestamp, FileExtension::pdf());

                let (asset_id, filename) = match save_asset(
                    &self.storage,
                    self.db.clone(),
                    self.room_id.room_id(),
                    Some(Self::NAMESPACE),
                    filename,
                    data,
                    ChunkFormat::Data,
                )
                .await
                {
                    Ok(asset_id) => asset_id,
                    Err(AssetError::AssetStorageExceeded) => {
                        ctx.ws_send(Error::StorageExceeded);
                        return Ok(());
                    }
                    Err(e) => {
                        let message =
                            format!("Failed to save whiteboard asset: {}", Report::from_error(e));

                        log::error!("{message}");
                        whatever!("{message}");
                    }
                };

                ctx.exchange_publish(
                    control::exchange::current_room_all_participants(self.room_id),
                    exchange::Event::PdfAsset(PdfAsset { filename, asset_id }),
                );

                Ok(())
            }
            // ignored events
            Event::Leaving
            | Event::RaiseHand
            | Event::LowerHand
            | Event::ParticipantJoined(_, _)
            | Event::ParticipantLeft(_)
            | Event::ParticipantUpdated(_, _)
            | Event::RoleUpdated(_) => Ok(()),
        }
    }

    async fn on_destroy(self, ctx: DestroyContext<'_>) {
        // FIXME: We can not save the PDF here as it potentially takes more than a few seconds to generate the PDF
        // and we hold the r3dlock in the destroy context.

        match ctx.cleanup_scope {
            CleanupScope::None => (),
            CleanupScope::Local => {
                self.cleanup_room(ctx.volatile.storage(), self.room_id)
                    .await
            }
            CleanupScope::Global => {
                self.cleanup_room(ctx.volatile.storage(), self.room_id)
                    .await;

                if self.room_id.breakout_room_id().is_some() {
                    self.cleanup_room(
                        ctx.volatile.storage(),
                        SignalingRoomId::new(self.room_id.room_id(), None),
                    )
                    .await;
                }
            }
        }
    }

    async fn build_params(
        init: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
        let spacedeck = init.settings_provider.get().spacedeck.clone();

        Ok(spacedeck)
    }
}

impl Whiteboard {
    /// Creates a new spacedeck space
    ///
    /// When spacedeck gets initialized here, this function will send the [`exchange::Event::Initialized`] to all
    /// participants in the room
    async fn create_space(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        match ctx.volatile.storage().try_start_init(self.room_id).await? {
            Some(state) => match state {
                InitState::Initializing => ctx.ws_send(Error::CurrentlyInitializing),
                InitState::Initialized(_) => ctx.ws_send(Error::AlreadyInitialized),
            },
            None => {
                let response = self
                    .client
                    .create_space(&self.room_id.to_string(), None)
                    .await?;

                let url = self.client.base_url.join(&format!(
                    "s/{hash}-{slug}",
                    hash = response.edit_hash,
                    slug = response.edit_slug
                ))?;

                let space_info = SpaceInfo {
                    id: response.id,
                    url,
                };

                ctx.volatile
                    .storage()
                    .set_initialized(self.room_id, space_info)
                    .await?;

                ctx.exchange_publish(
                    control::exchange::current_room_all_participants(self.room_id),
                    exchange::Event::Initialized,
                );
            }
        }
        Ok(())
    }

    async fn cleanup_room(
        &self,
        storage: &mut dyn WhiteboardStorage,
        signaling_room_id: SignalingRoomId,
    ) {
        let state = match storage.get_init_state(signaling_room_id).await {
            Ok(Some(state)) => state,
            Ok(None) => return,
            Err(e) => {
                log::error!(
                    "Failed to get state for the whiteboard module {}",
                    Report::from_error(e)
                );

                return;
            }
        };

        if let Err(e) = storage.delete_init_state(signaling_room_id).await {
            log::error!(
                "Failed to delete state for the whiteboard module {}",
                Report::from_error(e)
            )
        }

        if let InitState::Initialized(space_info) = state {
            if let Err(e) = self.client.delete_space(&space_info.id).await {
                log::error!(
                    "Failed to delete space from spacedeck {}",
                    Report::from_error(e)
                )
            }
        }
    }
}
