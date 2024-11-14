// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! # Shared Folder Module
//!
//! ## Functionality
//!
//! Allows sharing a link to an external service providing password-protected a shared folder

use std::sync::Arc;

use either::Either;
use opentalk_database::Db;
use opentalk_db_storage::events::shared_folders::EventSharedFolder;
use opentalk_signaling_core::{
    CleanupScope, DestroyContext, Event, InitContext, ModuleContext, SignalingModule,
    SignalingModuleError, SignalingModuleInitData, SignalingRoomId, VolatileStorage,
};
use opentalk_types_common::shared_folders::{SharedFolder as SharedFolderType, NAMESPACE};
use opentalk_types_signaling::ForRole as _;
use opentalk_types_signaling_shared_folder::event::SharedFolderEvent;
use snafu::Report;
use storage::SharedFolderStorage;

mod storage;

pub struct SharedFolder {
    room: SignalingRoomId,
    db: Arc<Db>,
}

impl SharedFolder {
    async fn cleanup_room(ctx: &mut DestroyContext<'_>, signaling_room_id: SignalingRoomId) {
        let storage = ctx.volatile.storage();

        if let Err(e) = storage
            .delete_shared_folder_initialized(signaling_room_id)
            .await
        {
            log::error!(
                "Failed to remove shared folder initialized flag on room destroy, {}",
                e
            );
        }
        if let Err(e) = storage.delete_shared_folder(signaling_room_id).await {
            log::error!(
                "Failed to remove shared folder on room destroy, {}",
                Report::from_error(e)
            );
        }
    }
}

trait SharedFolderStorageProvider {
    fn storage(&mut self) -> &mut dyn SharedFolderStorage;
}

impl SharedFolderStorageProvider for VolatileStorage {
    fn storage(&mut self) -> &mut dyn SharedFolderStorage {
        match self.as_mut() {
            Either::Left(v) => v,
            Either::Right(v) => v,
        }
    }
}

#[async_trait::async_trait(? Send)]
impl SignalingModule for SharedFolder {
    const NAMESPACE: &'static str = NAMESPACE;

    type Params = ();

    type Incoming = ();
    type Outgoing = SharedFolderEvent;
    type ExchangeMessage = ();

    type ExtEvent = ();

    type FrontendData = SharedFolderType;
    type PeerFrontendData = ();

    async fn init(
        ctx: InitContext<'_, Self>,
        _params: &Self::Params,
        _protocol: &'static str,
    ) -> Result<Option<Self>, SignalingModuleError> {
        Ok(Some(Self {
            room: ctx.room_id(),
            db: ctx.db().clone(),
        }))
    }

    async fn on_event(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        event: Event<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        match event {
            Event::Joined {
                control_data,
                frontend_data,
                participants: _,
            } => {
                if !ctx
                    .volatile
                    .storage()
                    .is_shared_folder_initialized(self.room)
                    .await?
                {
                    if let Some(event) = ctx
                        .volatile
                        .storage()
                        .get_event(self.room.room_id())
                        .await?
                    {
                        let mut conn = self.db.get_conn().await?;
                        if let Some(shared_folder) =
                            EventSharedFolder::get_for_event(&mut conn, event.id).await?
                        {
                            ctx.volatile
                                .storage()
                                .set_shared_folder(self.room, shared_folder.into())
                                .await?;
                        }
                    };
                    ctx.volatile
                        .storage()
                        .set_shared_folder_initialized(self.room)
                        .await?;
                }

                *frontend_data = ctx
                    .volatile
                    .storage()
                    .get_shared_folder(self.room)
                    .await?
                    .map(|f| f.for_role(control_data.role));
            }
            Event::Leaving => {}
            Event::RaiseHand => {}
            Event::LowerHand => {}
            Event::ParticipantJoined(_, _) => {}
            Event::ParticipantLeft(_) => {}
            Event::ParticipantUpdated(_, _) => {}
            Event::RoleUpdated(role) => {
                if let Some(shared_folder) =
                    ctx.volatile.storage().get_shared_folder(self.room).await?
                {
                    let update = SharedFolderEvent::Updated(shared_folder.for_role(role));
                    ctx.ws_send(update);
                }
            }
            Event::WsMessage(_) => {}
            Event::Exchange(_) => {}
            Event::Ext(_) => {}
        }

        Ok(())
    }

    async fn on_destroy(self, mut ctx: DestroyContext<'_>) {
        match ctx.cleanup_scope {
            CleanupScope::None => (),
            CleanupScope::Local => SharedFolder::cleanup_room(&mut ctx, self.room).await,
            CleanupScope::Global => {
                if self.room.breakout_room_id().is_some() {
                    SharedFolder::cleanup_room(
                        &mut ctx,
                        SignalingRoomId::new(self.room.room_id(), None),
                    )
                    .await
                }

                SharedFolder::cleanup_room(&mut ctx, self.room).await
            }
        }
    }

    async fn build_params(
        init: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
        Ok(init
            .shared_settings
            .load_full()
            .shared_folder
            .as_ref()
            .map(|_| ()))
    }
}
