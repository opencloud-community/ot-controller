// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{sync::Arc, time::Duration};

use livekit_api::{
    access_token::{AccessToken, VideoGrants},
    services::room::{CreateRoomOptions, RoomClient, UpdateParticipantOptions},
};
use livekit_protocol::{ParticipantPermission, TrackSource};
use opentalk_controller_settings::{LiveKitSettings, SharedSettings};
use opentalk_signaling_core::{
    DestroyContext, Event, InitContext, ModuleContext, SignalingModule, SignalingModuleError,
    SignalingModuleInitData, SignalingRoomId,
};
use opentalk_types_signaling::{ParticipantId, ParticipationKind, ParticipationVisibility, Role};
use opentalk_types_signaling_livekit::{
    command, event,
    state::{self, LiveKitState},
    Credentials,
};
use snafu::ResultExt;

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
    participation_kind: ParticipationKind,
    role: Role,
    params: Arc<LivekitParams>,
}

pub struct LivekitParams {
    shared_settings: SharedSettings,
    livekit_settings: LiveKitSettings,
    room_client: RoomClient,
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for Livekit {
    const NAMESPACE: &'static str = opentalk_types_signaling_livekit::NAMESPACE;

    type Params = Arc<LivekitParams>;
    type Incoming = command::LiveKitCommand;
    type Outgoing = event::LiveKitEvent;
    type ExchangeMessage = ();
    type ExtEvent = ();
    type FrontendData = state::LiveKitState;
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
            participation_kind: ctx.participant().kind(),
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
                self.participation_kind = control_data.participation_kind;
                let (room_name, access_token) = self
                    .create_room_and_access_token(
                        &mut ctx,
                        control_data.participation_kind.visibility(),
                    )
                    .await?;

                let service_url = match control_data.participation_kind {
                    ParticipationKind::User | ParticipationKind::Guest => None,
                    ParticipationKind::Sip | ParticipationKind::Recorder => {
                        let service_url = self.params.livekit_settings.service_url.clone();
                        let service_url_ws = if service_url.starts_with("http") {
                            service_url.replacen("http", "ws", 1)
                        } else {
                            service_url
                        };

                        Some(service_url_ws)
                    }
                };

                *frontend_data = Some(LiveKitState(Credentials {
                    room: room_name,
                    token: access_token,
                    public_url: self.params.livekit_settings.public_url.clone(),
                    service_url,
                }));

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
        let Some(livekit_settings) = &init.startup_settings.livekit else {
            return Ok(None);
        };

        let room_client = RoomClient::with_api_key(
            &livekit_settings.service_url,
            &livekit_settings.api_key,
            &livekit_settings.api_secret,
        );

        Ok(Some(Arc::new(LivekitParams {
            shared_settings: init.shared_settings.clone(),
            livekit_settings: livekit_settings.clone(),
            room_client,
        })))
    }
}

impl Livekit {
    async fn handle_command(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        command: command::LiveKitCommand,
    ) -> Result<(), SignalingModuleError> {
        match command {
            command::LiveKitCommand::CreateNewAccessToken => {
                let (room_name, access_token) = self
                    .create_room_and_access_token(&mut ctx, self.participation_kind.visibility())
                    .await?;

                let service_url = match self.participation_kind {
                    ParticipationKind::User | ParticipationKind::Guest => None,
                    ParticipationKind::Sip | ParticipationKind::Recorder => {
                        let service_url = self.params.livekit_settings.service_url.clone();
                        let service_url_ws = if service_url.starts_with("http") {
                            service_url.replacen("http", "ws", 1)
                        } else {
                            service_url
                        };

                        Some(service_url_ws)
                    }
                };

                ctx.ws_send(event::LiveKitEvent::State(LiveKitState(Credentials {
                    room: room_name,
                    token: access_token,
                    public_url: self.params.livekit_settings.public_url.clone(),
                    service_url,
                })));

                Ok(())
            }
            command::LiveKitCommand::ForceMute { participants } => {
                if !self.role.is_moderator() {
                    ctx.ws_send(event::LiveKitEvent::Error(
                        event::Error::InsufficientPermissions,
                    ));
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
            command::LiveKitCommand::GrantScreenSharePermission { participants } => {
                self.set_screenshare_permissions(ctx, participants, true)
                    .await
            }
            command::LiveKitCommand::RevokeScreenSharePermission { participants } => {
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
            ctx.ws_send(event::LiveKitEvent::Error(
                event::Error::InsufficientPermissions,
            ));
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
        visibility: ParticipationVisibility,
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
                ctx.ws_send(event::LiveKitEvent::Error(event::Error::LivekitUnavailable));
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
            hidden: visibility.is_hidden(),
            recorder: false,
        })
        .with_ttl(ACCESS_TOKEN_TTL)
        .to_jwt()
        .whatever_context::<&str, SignalingModuleError>("Failed to create livekit access-token")?;

        Ok((room.name, access_token))
    }
}
