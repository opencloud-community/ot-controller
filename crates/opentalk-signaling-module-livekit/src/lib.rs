// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::BTreeSet, sync::Arc, time::Duration};

use either::Either;
use futures::{
    stream::{self},
    StreamExt,
};
use livekit_api::{
    access_token::{AccessToken, VideoGrants},
    services::room::{CreateRoomOptions, RoomClient, UpdateParticipantOptions},
};
use livekit_protocol::{ParticipantPermission, TrackSource};
use opentalk_controller_settings::{LiveKitSettings, SharedSettings};
use opentalk_signaling_core::{
    control, CleanupScope, DestroyContext, Event, InitContext, ModuleContext, SignalingModule,
    SignalingModuleError, SignalingModuleInitData, SignalingRoomId, VolatileStorage,
};
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::{ParticipantId, ParticipationKind, ParticipationVisibility, Role};
use opentalk_types_signaling_livekit::{
    command::{self, UnrestrictedParticipants},
    event,
    state::{self, LiveKitState},
    Credentials, MicrophoneRestrictionState,
};
use snafu::ResultExt;
use storage::LivekitStorage;

mod exchange;
mod storage;

const PARALLEL_UPDATES: usize = 25;
const ACCESS_TOKEN_TTL: Duration = Duration::from_secs(32);
const LIVEKIT_MEDIA_SOURCES: [TrackSource; 4] = [
    TrackSource::Camera,
    TrackSource::Microphone,
    TrackSource::ScreenShare,
    TrackSource::ScreenShareAudio,
];

pub struct Livekit {
    room_id: SignalingRoomId,
    participant_id: ParticipantId,
    participation_kind: ParticipationKind,
    role: Role,
    params: Arc<LivekitParams>,
    token_identities: BTreeSet<String>,
}

pub struct LivekitParams {
    shared_settings: SharedSettings,
    livekit_settings: LiveKitSettings,
    room_client: RoomClient,
}

trait LivekitStorageProvider {
    fn storage(&mut self) -> &mut dyn LivekitStorage;
}

impl LivekitStorageProvider for VolatileStorage {
    fn storage(&mut self) -> &mut dyn LivekitStorage {
        match self.as_mut() {
            Either::Left(v) => v,
            Either::Right(v) => v,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for Livekit {
    const NAMESPACE: ModuleId = opentalk_types_signaling_livekit::MODULE_ID;

    type Params = Arc<LivekitParams>;
    type Incoming = command::LiveKitCommand;
    type Outgoing = event::LiveKitEvent;
    type ExchangeMessage = exchange::Message;
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
            token_identities: BTreeSet::default(),
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
                let (room_name, access_token, microphone_restriction_state) = self
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

                *frontend_data = Some(LiveKitState {
                    credentials: Credentials {
                        room: room_name,
                        token: access_token,
                        public_url: self.params.livekit_settings.public_url.clone(),
                        service_url,
                    },
                    microphone_restriction_state,
                });

                Ok(())
            }
            Event::WsMessage(command) => self.handle_command(ctx, command).await,
            Event::RoleUpdated(role) => {
                self.role = role;
                Ok(())
            }
            Event::Exchange(message) => {
                self.handle_exchange_message(ctx, message).await;
                Ok(())
            }
            Event::Leaving => {
                // Attempt to remove the existing livekit identities
                for identity in &self.token_identities {
                    if let Err(e) = self
                        .params
                        .room_client
                        .remove_participant(&self.room_id.to_string(), identity)
                        .await
                    {
                        // There is nothing we can do in the case of an error, just continue with the other identities.
                        //
                        // The most likely error here is an `not_found` error. However, we can't match the returned error
                        // types because livekit does not expose them.
                        log::debug!(
                            "Failed to remove participant identity {} from livekit in room {}: {}",
                            identity,
                            self.room_id,
                            e
                        )
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn on_destroy(self, ctx: DestroyContext<'_>) {
        match ctx.cleanup_scope {
            CleanupScope::None => (),
            CleanupScope::Local => self.cleanup_room(self.room_id).await,
            CleanupScope::Global => {
                if self.room_id.breakout_room_id().is_some() {
                    self.cleanup_room(SignalingRoomId::new(self.room_id.room_id(), None))
                        .await
                }

                self.cleanup_room(self.room_id).await
            }
        }
    }

    async fn build_params(
        init: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
        let LiveKitSettings {
            api_key,
            api_secret,
            service_url,
            ..
        } = &init.startup_settings.livekit;

        let room_client = RoomClient::with_api_key(service_url, api_key, api_secret);

        Ok(Some(Arc::new(LivekitParams {
            shared_settings: init.shared_settings.clone(),
            livekit_settings: init.startup_settings.livekit.clone(),
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
                let (room_name, access_token, _) = self
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

                ctx.ws_send(event::LiveKitEvent::Credentials(Credentials {
                    room: room_name,
                    token: access_token,
                    public_url: self.params.livekit_settings.public_url.clone(),
                    service_url,
                }));

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
                    let participant_id_str = participant_id.to_string();

                    let participant = match self
                        .params
                        .room_client
                        .get_participant(&room, &participant_id_str)
                        .await
                    {
                        Ok(p) => p,
                        Err(e) => {
                            log::error!("Failed fetch participant room={room} participant={participant_id_str}, {e}");
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
                            .mute_published_track(&room, &participant_id_str, &track.sid, true)
                            .await
                        {
                            log::error!("Failed to mute track room={room} participant={participant_id_str} track-id={}, {e}", track.sid);
                        }
                    }

                    ctx.exchange_publish(
                        control::exchange::current_room_by_participant_id(
                            self.room_id,
                            participant_id,
                        ),
                        exchange::Message::ForceMuted {
                            moderator: self.participant_id,
                        },
                    );
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
            command::LiveKitCommand::EnableMicrophoneRestrictions(UnrestrictedParticipants {
                unrestricted_participants,
            }) => {
                self.set_microphone_permissions(ctx, unrestricted_participants, true)
                    .await
            }
            command::LiveKitCommand::DisableMicrophoneRestrictions => {
                self.set_microphone_permissions(ctx, vec![], false).await
            }
            command::LiveKitCommand::RequestPopoutStreamAccessToken => {
                self.create_popout_stream_access_token(&mut ctx).await
            }
        }
    }

    async fn handle_exchange_message(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        message: exchange::Message,
    ) {
        match message {
            exchange::Message::MicrophoneRestrictionsEnabled(microphone_restrictions_enabled) => {
                ctx.ws_send(event::LiveKitEvent::MicrophoneRestrictionsEnabled(
                    microphone_restrictions_enabled,
                ));
            }
            exchange::Message::MicrophoneRestrictionsDisabled => {
                ctx.ws_send(event::LiveKitEvent::MicrophoneRestrictionsDisabled);
            }
            exchange::Message::ForceMuted { moderator } => {
                ctx.ws_send(event::LiveKitEvent::ForceMuted { moderator });
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
        let all_participants = self
            .params
            .room_client
            .list_participants(&room)
            .await
            .whatever_context::<&str, SignalingModuleError>(
            "Failed to list livekit participants",
        )?;

        let participant_ids: Vec<String> =
            participants.into_iter().map(|id| id.to_string()).collect();
        let affected_participants = all_participants
            .into_iter()
            .filter(|p| participant_ids.contains(&p.identity))
            .collect::<Vec<_>>();

        let screenshare_source_number = TrackSource::ScreenShare as i32;
        let screenshare_audio_source_number = TrackSource::ScreenShareAudio as i32;

        self.update_participants_permission(
            affected_participants,
            &[screenshare_source_number, screenshare_audio_source_number],
            grant,
            &room,
        )
        .await;

        Ok(())
    }

    async fn set_microphone_permissions(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        unrestricted_participants: Vec<ParticipantId>,
        restrict: bool,
    ) -> Result<(), SignalingModuleError> {
        if !self.role.is_moderator() {
            ctx.ws_send(event::LiveKitEvent::Error(
                event::Error::InsufficientPermissions,
            ));
            return Ok(());
        }

        let room = self.room_id.to_string();
        let mut participants = self
            .params
            .room_client
            .list_participants(&room)
            .await
            .whatever_context::<&str, SignalingModuleError>(
            "Failed to list livekit participants",
        )?;

        if restrict {
            ctx.volatile
                .storage()
                .set_microphone_restriction_allow_list(
                    self.room_id.room_id(),
                    &unrestricted_participants[..],
                )
                .await?;

            ctx.exchange_publish(
                control::exchange::current_room_all_participants(self.room_id),
                exchange::Message::MicrophoneRestrictionsEnabled(UnrestrictedParticipants {
                    unrestricted_participants: unrestricted_participants.clone(),
                }),
            );

            let allowed_ids = unrestricted_participants
                .into_iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>();
            participants.retain(|part| !allowed_ids.contains(&part.identity));
        } else {
            ctx.volatile
                .storage()
                .clear_microphone_restriction(self.room_id.room_id())
                .await?;

            ctx.exchange_publish(
                control::exchange::current_room_all_participants(self.room_id),
                exchange::Message::MicrophoneRestrictionsDisabled,
            );
        }

        let microphone_source_number = TrackSource::Microphone as i32;

        self.update_participants_permission(
            participants,
            &[microphone_source_number],
            !restrict,
            &room,
        )
        .await;

        Ok(())
    }

    async fn update_participants_permission(
        &self,
        participants: Vec<livekit_protocol::ParticipantInfo>,
        source_numbers: &[i32],
        grant: bool,
        room: &str,
    ) {
        stream::iter(participants)
            .map(|participant| {
                self.update_single_participant_permission(participant, source_numbers, grant, room)
            })
            .buffer_unordered(PARALLEL_UPDATES)
            .collect::<Vec<_>>()
            .await;
    }

    async fn update_single_participant_permission(
        &self,
        participant: livekit_protocol::ParticipantInfo,
        source_numbers: &[i32],
        grant: bool,
        room: &str,
    ) {
        let mut can_publish_sources = participant
            .permission
            .map(|p| p.can_publish_sources)
            .unwrap_or_else(|| {
                LIVEKIT_MEDIA_SOURCES
                    .map(|s: TrackSource| s as i32)
                    .to_vec()
            });

        for source_number in source_numbers.iter() {
            Self::update_publish_sources(&mut can_publish_sources, *source_number, grant)
        }

        if let Err(e) = self
            .params
            .room_client
            .update_participant(
                room,
                &participant.identity,
                UpdateParticipantOptions {
                    permission: Some(ParticipantPermission {
                        can_subscribe: true,
                        can_publish: true,
                        can_publish_data: false,
                        can_publish_sources,
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
                "Failed to update participant room={room} participant={}, {e}",
                participant.identity
            );
        }
    }

    fn update_publish_sources(can_publish_sources: &mut Vec<i32>, source: i32, grant: bool) {
        if grant {
            if !can_publish_sources.contains(&source) {
                can_publish_sources.push(source);
            }
        } else {
            can_publish_sources.retain(|&x| x != source);
        }
    }

    /// Create Room and AccessToken
    ///
    /// Returns (RoomName, AccessToken)
    async fn create_room_and_access_token(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        visibility: ParticipationVisibility,
    ) -> Result<(String, String, MicrophoneRestrictionState), SignalingModuleError> {
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

        let microphone_restriction_state = ctx
            .volatile
            .storage()
            .get_microphone_restriction_state(self.room_id.room_id())
            .await?;

        let allow_screenshare = !self
            .params
            .shared_settings
            .load()
            .defaults
            .screen_share_requires_permission;

        let mut available_sources = LIVEKIT_MEDIA_SOURCES.to_vec();
        if !self.role.is_moderator() {
            if let MicrophoneRestrictionState::Enabled {
                unrestricted_participants,
            } = &microphone_restriction_state
            {
                if !unrestricted_participants.contains(&self.participant_id) {
                    available_sources.retain(|s| s != &TrackSource::Microphone);
                }
            }
        }

        if !allow_screenshare {
            available_sources
                .retain(|s| s != &TrackSource::ScreenShare && s != &TrackSource::ScreenShareAudio);
        };

        let can_publish_sources = available_sources
            .into_iter()
            .map(|s| TrackSource::as_str_name(&s).to_lowercase())
            .collect();

        let identity = self.participant_id.to_string();

        let access_token = AccessToken::with_api_key(
            &self.params.livekit_settings.api_key,
            &self.params.livekit_settings.api_secret,
        )
        .with_name(&identity)
        .with_identity(&identity)
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

        self.token_identities.insert(identity);

        Ok((room.name, access_token, microphone_restriction_state))
    }

    async fn cleanup_room(&self, signaling_room_id: SignalingRoomId) {
        if let Err(e) = self
            .params
            .room_client
            .delete_room(&signaling_room_id.to_string())
            .await
        {
            log::error!("Failed to destroy livekit room {e}");
        }
    }

    async fn create_popout_stream_access_token(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        let identity = format!(
            "{}-popout{}",
            self.participant_id,
            self.token_identities.len()
        );

        let token = AccessToken::with_api_key(
            &self.params.livekit_settings.api_key,
            &self.params.livekit_settings.api_secret,
        )
        .with_name(&identity)
        .with_identity(&identity)
        .with_grants(VideoGrants {
            room_create: false,
            room_list: false,
            room_record: false,
            room_admin: false,
            room_join: true,
            room: self.room_id.to_string(),
            can_publish: false,
            can_subscribe: true,
            can_publish_data: false,
            can_publish_sources: vec![],
            can_update_own_metadata: false,
            ingress_admin: false,
            hidden: true,
            recorder: false,
        })
        .with_ttl(ACCESS_TOKEN_TTL)
        .to_jwt()
        .whatever_context::<&str, SignalingModuleError>("Failed to create livekit access-token")?;

        self.token_identities.insert(identity);

        ctx.ws_send(event::LiveKitEvent::PopoutStreamAccessToken { token });

        Ok(())
    }
}
