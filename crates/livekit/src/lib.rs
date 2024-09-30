// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{sync::Arc, time::Duration};

use livekit_api::{
    access_token::{AccessToken, VideoGrants},
    services::room::{CreateRoomOptions, RoomClient},
};
use livekit_protocol::TrackSource;
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

pub struct Livekit {
    room_id: SignalingRoomId,
    participant_id: ParticipantId,
    role: Role,
    params: Arc<LivekitParams>,
}

pub struct LivekitParams {
    settings: LivekitSettings,
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
            Event::WsMessage(commands::Command::CreateNewAccessToken) => {
                let (room_name, access_token) = self.create_room_and_access_token(&mut ctx).await?;

                ctx.ws_send(events::Event::Credentials(events::Credentials {
                    room: room_name,
                    token: access_token,
                }));

                Ok(())
            }
            Event::WsMessage(commands::Command::ForceMute { participants }) => {
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
        let Some(settings) =
            LivekitSettings::extract(&init.startup_settings).context(ConfigSnafu)?
        else {
            return Ok(None);
        };

        let room_client =
            RoomClient::with_api_key(&settings.url, &settings.api_key, &settings.api_secret);

        Ok(Some(Arc::new(LivekitParams {
            settings,
            room_client,
        })))
    }
}

impl Livekit {
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

        let participant_id = self.participant_id.to_string();

        let access_token = AccessToken::with_api_key(
            &self.params.settings.api_key,
            &self.params.settings.api_secret,
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
            can_publish_sources: vec![],
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
