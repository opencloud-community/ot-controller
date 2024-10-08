// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{sync::Arc, time::Duration};

use livekit_api::{
    access_token::{AccessToken, VideoGrants},
    services::room::{CreateRoomOptions, RoomClient, UpdateParticipantOptions},
};
use livekit_protocol::{ParticipantPermission, TrackSource};
use opentalk_controller_settings::SharedSettings;
use opentalk_signaling_core::{
    ConfigSnafu, DestroyContext, Event, InitContext, ModuleContext, SignalingModule,
    SignalingModuleError, SignalingModuleInitData, SignalingRoomId,
};
use opentalk_types_signaling::{ParticipantId, Role, SignalingModuleFrontendData};
use settings::LivekitSettings;
use snafu::ResultExt;

mod commands;
mod events;
mod settings;

const ACCESS_TOKEN_TTL: Duration = Duration::from_secs(32);
const SOURCES_WITH_SCREENSHARE: [TrackSource; 4] = [
    TrackSource::Camera,
    TrackSource::Microphone,
    TrackSource::ScreenShare,
    TrackSource::ScreenShareAudio,
];
const SOURCES_WITHOUT_SCREENSHARE: [TrackSource; 2] =
    [TrackSource::Camera, TrackSource::Microphone];

pub struct Livekit {
    room_id: SignalingRoomId,
    participant_id: ParticipantId,
    role: Role,
    params: Arc<LivekitParams>,
}

pub struct LivekitParams {
    shared_settings: SharedSettings,
    livekit_settings: LivekitSettings,
    room_client: RoomClient,
}

impl SignalingModuleFrontendData for events::Credentials {
    const NAMESPACE: Option<&'static str> = Some(Livekit::NAMESPACE);
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for Livekit {
    const NAMESPACE: &'static str = "livekit";

    type Params = Arc<LivekitParams>;
    type Incoming = commands::Command;
    type Outgoing = events::Event;
    type ExchangeMessage = ();
    type ExtEvent = ();
    type FrontendData = events::Credentials;
    type PeerFrontendData = ();

    async fn init(
        ctx: InitContext<'_, Self>,
        params: &Self::Params,
        _: &'static str,
    ) -> Result<Option<Self>, SignalingModuleError> {
        Ok(Some(Self {
            room_id: ctx.room_id(),
            participant_id: ctx.participant_id(),
            role: ctx.role(),
            params: params.clone(),
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
                let (room_name, access_token) = self.create_room_and_access_token(&mut ctx).await?;

                *frontend_data = Some(events::Credentials {
                    room: room_name,
                    token: access_token,
                });

                Ok(())
            }
            Event::WsMessage(command) => self.handle_command(ctx, command).await,
            Event::RoleUpdated(role) => {
                self.role = role;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn on_destroy(self, ctx: DestroyContext<'_>) {
        if !ctx.destroy_room() {
            return;
        }

        if let Err(e) = self
            .params
            .room_client
            .delete_room(&self.room_id.to_string())
            .await
        {
            log::error!("Failed to destroy livekit room {e}");
        }
    }

    async fn build_params(
        init: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
        let Some(livekit_settings) =
            LivekitSettings::extract(&init.startup_settings).context(ConfigSnafu)?
        else {
            return Ok(None);
        };

        let room_client = RoomClient::with_api_key(
            &livekit_settings.url,
            &livekit_settings.api_key,
            &livekit_settings.api_secret,
        );

        Ok(Some(Arc::new(LivekitParams {
            shared_settings: init.shared_settings.clone(),
            livekit_settings,
            room_client,
        })))
    }
}

impl Livekit {
    async fn handle_command(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        command: commands::Command,
    ) -> Result<(), SignalingModuleError> {
        match command {
            commands::Command::CreateNewAccessToken => {
                let (room_name, access_token) = self.create_room_and_access_token(&mut ctx).await?;

                ctx.ws_send(events::Event::Credentials(events::Credentials {
                    room: room_name,
                    token: access_token,
                }));

                Ok(())
            }
            commands::Command::ForceMute { participants } => {
                if !self.role.is_moderator() {
                    ctx.ws_send(events::Event::Error(events::Error::InsufficientPermissions));
                    return Ok(());
                }

                let room = self.room_id.to_string();

                for participant_id in participants {
                    let participant_id = participant_id.to_string();

                    let participant = match self
                        .params
                        .room_client
                        .get_participant(&room, &participant_id)
                        .await
                    {
                        Ok(p) => p,
                        Err(e) => {
                            log::error!("Failed fetch participant room={room} participant={participant_id}, {e}");
                            continue;
                        }
                    };

                    for track in participant.tracks {
                        if track.source != TrackSource::Microphone as i32 {
                            // Don't mute non-microphone tracks
                            continue;
                        }

                        if let Err(e) = self
                            .params
                            .room_client
                            .mute_published_track(&room, &participant_id, &track.sid, true)
                            .await
                        {
                            log::error!("Failed to mute track room={room} participant={participant_id} track-id={}, {e}", track.sid);
                        }
                    }
                }

                Ok(())
            }
            commands::Command::GrantScreenSharePermission { participants } => {
                self.set_screenshare_permissions(ctx, participants, true)
                    .await
            }
            commands::Command::RevokeScreenSharePermission { participants } => {
                self.set_screenshare_permissions(ctx, participants, false)
                    .await
            }
        }
    }

    async fn set_screenshare_permissions(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        participants: Vec<ParticipantId>,
        grant: bool,
    ) -> Result<(), SignalingModuleError> {
        if !self.role.is_moderator() {
            ctx.ws_send(events::Event::Error(events::Error::InsufficientPermissions));
            return Ok(());
        }

        let room = self.room_id.to_string();

        let can_publish_sources = if grant {
            SOURCES_WITH_SCREENSHARE.map(|s| s as i32).to_vec()
        } else {
            SOURCES_WITHOUT_SCREENSHARE.map(|s| s as i32).to_vec()
        };

        for participant_id in participants {
            if let Err(e) = self
                .params
                .room_client
                .update_participant(
                    &room,
                    &participant_id.to_string(),
                    UpdateParticipantOptions {
                        permission: Some(ParticipantPermission {
                            can_subscribe: true,
                            can_publish: true,
                            can_publish_data: false,
                            can_publish_sources: can_publish_sources.clone(),
                            hidden: false,
                            can_update_metadata: false,
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                )
                .await
            {
                log::error!(
                    "Failed to update participant room={room} participant={participant_id}, {e}",
                );
            }
        }

        Ok(())
    }

    /// Create Room and AccessToken
    ///
    /// Returns (RoomName, AccessToken)
    async fn create_room_and_access_token(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(String, String), SignalingModuleError> {
        let res = self
            .params
            .room_client
            .create_room(&self.room_id.to_string(), CreateRoomOptions::default())
            .await
            .whatever_context::<&str, SignalingModuleError>("Failed to create livekit room");

        let room = match res {
            Ok(room) => room,
            Err(e) => {
                ctx.ws_send(events::Event::Error(events::Error::LivekitUnavailable));
                return Err(e);
            }
        };

        let allow_screenshare = !self
            .params
            .shared_settings
            .load()
            .defaults
            .screen_share_requires_permission;

        let can_publish_sources = if allow_screenshare {
            SOURCES_WITH_SCREENSHARE
                .map(|s| TrackSource::as_str_name(&s).to_lowercase())
                .to_vec()
        } else {
            SOURCES_WITHOUT_SCREENSHARE
                .map(|s| TrackSource::as_str_name(&s).to_lowercase())
                .to_vec()
        };

        let participant_id = self.participant_id.to_string();

        let access_token = AccessToken::with_api_key(
            &self.params.livekit_settings.api_key,
            &self.params.livekit_settings.api_secret,
        )
        .with_name(&participant_id)
        .with_identity(&participant_id)
        .with_grants(VideoGrants {
            room_create: false,
            room_list: false,
            room_record: false,
            room_admin: false,
            room_join: true,
            room: room.name.clone(),
            can_publish: true,
            can_subscribe: true,
            can_publish_data: false,
            can_publish_sources,
            can_update_own_metadata: false,
            ingress_admin: false,
            hidden: false,
            recorder: false,
        })
        .with_ttl(ACCESS_TOKEN_TTL)
        .to_jwt()
        .whatever_context::<&str, SignalingModuleError>("Failed to create livekit access-token")?;

        Ok((room.name, access_token))
    }
}
