// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! # Shared Folder Module
//!
//! ## Functionality
//!
//! Allows sharing a link to an external service providing password-protected a shared folder

use std::sync::Arc;

use opentalk_database::Db;
use opentalk_db_storage::events::shared_folders::EventSharedFolder;
use opentalk_signaling_core::{
    control::storage::ControlStorage as _, DestroyContext, Event, InitContext, ModuleContext,
    SignalingModule, SignalingModuleError, SignalingModuleInitData, SignalingRoomId,
};
use opentalk_types::{
    common::shared_folder::SharedFolder as SharedFolderType,
    signaling::shared_folder::{event::SharedFolderEvent, NAMESPACE},
};
use snafu::Report;
use storage::SharedFolderStorage as _;

mod storage;

pub struct SharedFolder {
    room: SignalingRoomId,
    db: Arc<Db>,
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
                if !storage::is_shared_folder_initialized(ctx.redis_conn(), self.room).await? {
                    if let Some(event) = ctx.redis_conn().get_event(self.room.room_id()).await? {
                        let mut conn = self.db.get_conn().await?;
                        if let Some(shared_folder) =
                            EventSharedFolder::get_for_event(&mut conn, event.id).await?
                        {
                            storage::set_shared_folder(
                                ctx.redis_conn(),
                                self.room,
                                shared_folder.into(),
                            )
                            .await?;
                        }
                    };
                    ctx.redis_conn()
                        .set_shared_folder_initialized(self.room)
                        .await?;
                }

                *frontend_data = storage::get_shared_folder(ctx.redis_conn(), self.room)
                    .await?
                    .map(|f| f.for_signaling_role(control_data.role));
            }
            Event::Leaving => {}
            Event::RaiseHand => {}
            Event::LowerHand => {}
            Event::ParticipantJoined(_, _) => {}
            Event::ParticipantLeft(_) => {}
            Event::ParticipantUpdated(_, _) => {}
            Event::RoleUpdated(role) => {
                if let Some(shared_folder) =
                    storage::get_shared_folder(ctx.redis_conn(), self.room).await?
                {
                    let update = SharedFolderEvent::Updated(shared_folder.for_signaling_role(role));
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
        // ==== Cleanup room ====
        if ctx.destroy_room() {
            if let Err(e) =
                storage::delete_shared_folder_initialized(ctx.redis_conn(), self.room).await
            {
                log::error!(
                    "Failed to remove shared folder initialized flag on room destroy, {}",
                    e
                );
            }
            if let Err(e) = storage::delete_shared_folder(ctx.redis_conn(), self.room).await {
                log::error!(
                    "Failed to remove shared folder on room destroy, {}",
                    Report::from_error(e)
                );
            }
        }
    }

    async fn build_params(
        init: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
        if init.shared_settings.load_full().shared_folder.is_some() {
            Ok(Some(()))
        } else {
            log::warn!("Skipping the SharedFolder module as none is specified in the config");
            Ok(None)
        }
    }
}
