// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::Duration,
};

use either::Either;
use livekit_api::{
    access_token::{AccessToken, VideoGrants},
    services::room::{CreateRoomOptions, RoomClient},
};
use livekit_protocol::TrackSource;
use opentalk_controller_settings::LiveKitSettings;
use opentalk_signaling_core::{
    control, CleanupScope, DestroyContext, Event as SignalingEvent, InitContext, ModuleContext,
    SignalingModule, SignalingModuleError, SignalingModuleInitData, SignalingRoomId,
    VolatileStorage,
};
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_subroom_audio::{
    command::Command,
    event::{
        Error, Event, ParticipantsInvited, WhisperAccepted, WhisperGroupOutgoing, WhisperInvite,
        WhisperParticipantInfo, WhisperToken,
    },
    state::{WhisperGroup, WhisperState},
    whisper_id::WhisperId,
    NAMESPACE,
};
use snafu::ResultExt;
use storage::SubroomAudioStorage;

mod exchange;
mod storage;

const ACCESS_TOKEN_TTL: Duration = Duration::from_secs(32);

pub struct SubroomAudio {
    room_id: SignalingRoomId,
    params: Arc<SubroomAudioParams>,
    participant_id: ParticipantId,
}

pub struct SubroomAudioParams {
    settings: LiveKitSettings,
    room_client: RoomClient,
}

trait SubroomAudioStorageProvider {
    fn storage(&mut self) -> &mut dyn SubroomAudioStorage;
}

impl SubroomAudioStorageProvider for VolatileStorage {
    fn storage(&mut self) -> &mut dyn SubroomAudioStorage {
        match self.as_mut() {
            Either::Left(v) => v,
            Either::Right(v) => v,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for SubroomAudio {
    const NAMESPACE: &'static str = NAMESPACE;

    type Params = Arc<SubroomAudioParams>;
    type Incoming = Command;
    type Outgoing = Event;
    type ExchangeMessage = exchange::Message;
    type ExtEvent = ();
    type FrontendData = ();
    type PeerFrontendData = ();

    async fn init(
        ctx: InitContext<'_, Self>,
        params: &Self::Params,
        _: &'static str,
    ) -> Result<Option<Self>, SignalingModuleError> {
        Ok(Some(Self {
            room_id: ctx.room_id(),
            participant_id: ctx.participant_id(),
            params: params.clone(),
        }))
    }

    async fn on_event(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        event: SignalingEvent<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        match event {
            SignalingEvent::Leaving => {
                self.leave_all_whisper_groups(&mut ctx).await?;
            }
            SignalingEvent::WsMessage(cmd) => self.handle_command(&mut ctx, cmd).await?,
            SignalingEvent::Exchange(message) => {
                self.handle_exchange_message(&mut ctx, message).await?
            }
            SignalingEvent::Ext(_)
            | SignalingEvent::Joined { .. }
            | SignalingEvent::RaiseHand
            | SignalingEvent::LowerHand
            | SignalingEvent::RoleUpdated(_)
            | SignalingEvent::ParticipantJoined(_, _)
            | SignalingEvent::ParticipantLeft(_)
            | SignalingEvent::ParticipantUpdated(_, _) => (),
        }

        Ok(())
    }

    async fn on_destroy(self, mut ctx: DestroyContext<'_>) {
        match ctx.cleanup_scope {
            CleanupScope::None => (),
            CleanupScope::Local => {
                self.cleanup_whisper_groups(&mut ctx, self.room_id).await;
            }
            CleanupScope::Global => {
                if self.room_id.breakout_room_id().is_some() {
                    self.cleanup_whisper_groups(
                        &mut ctx,
                        SignalingRoomId::new(self.room_id.room_id(), None),
                    )
                    .await;
                }

                self.cleanup_whisper_groups(&mut ctx, self.room_id).await;
            }
        }
    }

    async fn build_params(
        init: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
        match &init.startup_settings.subroom_audio {
            Some(config) if config.enable_whisper => {}
            _ => return Ok(None),
        };

        let livekit_settings = &init.startup_settings.livekit;

        let room_client = RoomClient::with_api_key(
            &livekit_settings.service_url,
            &livekit_settings.api_key,
            &livekit_settings.api_secret,
        );

        Ok(Some(Arc::new(SubroomAudioParams {
            settings: livekit_settings.clone(),
            room_client,
        })))
    }
}

impl SubroomAudio {
    async fn handle_command(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        cmd: Command,
    ) -> Result<(), SignalingModuleError> {
        match cmd {
            Command::CreateWhisperGroup {
                participant_ids: participants,
            } => {
                self.create_whisper_group(ctx, participants).await?;
            }
            Command::InviteToWhisperGroup(targets) => {
                self.invite_to_whisper_group(ctx, targets.whisper_id, &targets.participant_ids)
                    .await?;
            }
            Command::KickWhisperParticipants(target) => {
                self.kick_participants(ctx, target.whisper_id, &target.participant_ids)
                    .await?;
            }
            Command::AcceptWhisperInvite { whisper_id } => {
                self.accept_whisper_invite(ctx, whisper_id).await?;
            }
            Command::DeclineWhisperInvite { whisper_id } => {
                let Some(whisper_group) = self.get_whisper_group(ctx, whisper_id).await? else {
                    return Ok(());
                };

                ctx.volatile
                    .storage()
                    .remove_participant(self.room_id, whisper_id, self.participant_id)
                    .await?;

                let whisper_declined = exchange::Message::WhisperDeclined(WhisperParticipantInfo {
                    whisper_id,
                    participant_id: self.participant_id,
                });

                self.notify_other_participants(
                    ctx,
                    whisper_group.participants.keys(),
                    whisper_declined,
                );
            }
            Command::LeaveWhisperGroup { whisper_id } => {
                let Some(whisper_group) = self.get_whisper_group(ctx, whisper_id).await? else {
                    return Ok(());
                };

                self.leave_whisper_group(ctx, whisper_group).await?;
            }
        }

        Ok(())
    }

    async fn handle_exchange_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        message: exchange::Message,
    ) -> Result<(), SignalingModuleError> {
        match message {
            exchange::Message::WhisperInvite(whisper_group) => {
                ctx.ws_send(Event::WhisperInvite(whisper_group));
            }
            exchange::Message::ParticipantsInvited(participants_invited) => {
                ctx.ws_send(Event::ParticipantsInvited(participants_invited));
            }
            exchange::Message::WhisperAccepted(whisper_accepted) => {
                ctx.ws_send(Event::WhisperInviteAccepted(whisper_accepted));
            }
            exchange::Message::WhisperDeclined(whisper_declined) => {
                ctx.ws_send(Event::WhisperInviteDeclined(whisper_declined));
            }
            exchange::Message::LeftWhisperGroup(whisper_declined) => {
                ctx.ws_send(Event::LeftWhisperGroup(whisper_declined));
            }
            exchange::Message::Kicked(whisper_id) => {
                if let Some(whisper_group) = self.get_whisper_group(ctx, whisper_id).await? {
                    self.leave_whisper_group(ctx, whisper_group).await?;
                }

                ctx.ws_send(Event::Kicked { whisper_id })
            }
        }

        Ok(())
    }

    async fn leave_whisper_group(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        whisper_group: WhisperGroup,
    ) -> Result<(), SignalingModuleError> {
        let has_token = whisper_group.has_accepted(&self.participant_id);

        if has_token {
            // This errors with a `not_found` when the participant already left or never joined. The frontend client
            // might leave the livekit room with the participant before we attempt to remove them. We can't match the
            // livekit error because the inner error types are not exposed. Simply ignore potential errors for now.
            let _ = self
                .params
                .room_client
                .remove_participant(
                    &whisper_group.whisper_id.to_string(),
                    &self.participant_id.to_string(),
                )
                .await;
        }

        let group_deleted = ctx
            .volatile
            .storage()
            .remove_participant(self.room_id, whisper_group.whisper_id, self.participant_id)
            .await?;

        if group_deleted {
            self.destroy_whisper_room(whisper_group.whisper_id).await;
        } else {
            let left_group = exchange::Message::LeftWhisperGroup(WhisperParticipantInfo {
                whisper_id: whisper_group.whisper_id,
                participant_id: self.participant_id,
            });

            self.notify_other_participants(ctx, whisper_group.participants.keys(), left_group);
        }

        Ok(())
    }

    async fn get_whisper_group(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        whisper_id: WhisperId,
    ) -> Result<Option<WhisperGroup>, SignalingModuleError> {
        let participants = match ctx
            .volatile
            .storage()
            .get_whisper_group(self.room_id, whisper_id)
            .await
        {
            Ok(participants) => participants,
            Err(err) => {
                log::debug!(
                    "Failed to get whisper group {} for room {}: {}",
                    whisper_id,
                    self.room_id,
                    err
                );

                ctx.ws_send(Event::Error(Error::InvalidWhisperId));
                return Ok(None);
            }
        };

        if !participants
            .iter()
            .any(|(participant_id, _)| participant_id == &self.participant_id)
        {
            ctx.ws_send(Event::Error(Error::NotInvited));
            return Ok(None);
        }

        Ok(Some(WhisperGroup {
            whisper_id,
            participants,
        }))
    }

    fn notify_other_participants<'a>(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participants: impl Iterator<Item = &'a ParticipantId>,
        message: exchange::Message,
    ) {
        for participant_id in participants {
            if participant_id == &self.participant_id {
                continue;
            }

            ctx.exchange_publish(
                control::exchange::current_room_by_participant_id(self.room_id, *participant_id),
                message.clone(),
            );
        }
    }

    async fn create_whisper_group(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participants: BTreeSet<ParticipantId>,
    ) -> Result<(), SignalingModuleError> {
        if !self.participant_targets_valid(ctx, &participants).await? {
            return Ok(());
        }

        let whisper_id = WhisperId::generate();

        let token = match self.create_room_and_access_token(whisper_id).await {
            Ok(token) => token,
            Err(e) => {
                ctx.ws_send(Event::Error(Error::LivekitUnavailable));
                return Err(e);
            }
        };

        let mut whisper_participants = participants
            .into_iter()
            .map(|participant_id| (participant_id, WhisperState::default()))
            .collect::<BTreeMap<_, _>>();

        whisper_participants.insert(self.participant_id, WhisperState::Creator);

        ctx.volatile
            .storage()
            .create_whisper_group(self.room_id, whisper_id, &whisper_participants)
            .await?;

        let group: WhisperGroupOutgoing = WhisperGroup {
            whisper_id,
            participants: whisper_participants.clone(),
        }
        .into();

        let invite = WhisperInvite {
            issuer: self.participant_id,
            group: group.clone(),
        };

        self.notify_other_participants(
            ctx,
            whisper_participants.keys(),
            exchange::Message::WhisperInvite(invite.clone()),
        );

        ctx.ws_send(Event::WhisperGroupCreated { token, group });

        Ok(())
    }

    async fn invite_to_whisper_group(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        whisper_id: WhisperId,
        participants: &BTreeSet<ParticipantId>,
    ) -> Result<(), SignalingModuleError> {
        let Some(mut whisper_group) = self.get_whisper_group(ctx, whisper_id).await? else {
            return Ok(());
        };

        if !self.participant_targets_valid(ctx, participants).await? {
            return Ok(());
        }

        if !self.is_group_creator(&whisper_group) {
            ctx.ws_send(Event::Error(Error::InsufficientPermissions));
            return Ok(());
        }

        // filter out participants that are already in this group
        let new_participants = participants
            .iter()
            .filter(|participant_id| !whisper_group.contains(participant_id))
            .map(|participant_id| (*participant_id, WhisperState::Invited))
            .collect::<BTreeMap<_, _>>();

        ctx.volatile
            .storage()
            .add_participants(self.room_id, whisper_id, &new_participants)
            .await?;

        let original_participants = whisper_group.participants.clone();

        whisper_group.participants.extend(new_participants.iter());

        let invite = WhisperInvite {
            issuer: self.participant_id,
            group: whisper_group.into(),
        };

        self.notify_other_participants(
            ctx,
            new_participants.keys(),
            exchange::Message::WhisperInvite(invite),
        );

        let participants_invited = ParticipantsInvited {
            whisper_id,
            participant_ids: new_participants.keys().copied().collect(),
        };

        // This is only send to the original whisper group participants
        self.notify_other_participants(
            ctx,
            original_participants.keys(),
            exchange::Message::ParticipantsInvited(participants_invited.clone()),
        );

        ctx.ws_send(Event::ParticipantsInvited(participants_invited));

        Ok(())
    }

    async fn kick_participants(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        whisper_id: WhisperId,
        targets: &BTreeSet<ParticipantId>,
    ) -> Result<(), SignalingModuleError> {
        let Some(whisper_group) = self.get_whisper_group(ctx, whisper_id).await? else {
            return Ok(());
        };

        if !self.is_group_creator(&whisper_group) {
            ctx.ws_send(Event::Error(Error::InsufficientPermissions));
            return Ok(());
        }

        if !self.participant_targets_valid(ctx, targets).await? {
            return Ok(());
        }

        let mut participants_to_kick = whisper_group.participants;

        participants_to_kick.retain(|participant_id, _| targets.contains(participant_id));

        self.notify_other_participants(
            ctx,
            participants_to_kick.keys(),
            exchange::Message::Kicked(whisper_id),
        );

        Ok(())
    }

    /// Checks if the provided participant list is valid
    ///
    /// Sends out appropriate WebSocket messages if participants do not exist in this room
    async fn participant_targets_valid(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participants: &BTreeSet<ParticipantId>,
    ) -> Result<bool, SignalingModuleError> {
        if participants.is_empty() {
            ctx.ws_send(Event::Error(Error::EmptyParticipantList));
            return Ok(false);
        }

        let room_participants = ctx
            .volatile
            .storage()
            .get_all_participants(self.room_id)
            .await?;

        let invalid_participants: Vec<_> = participants
            .difference(&room_participants)
            .copied()
            .collect();

        if !invalid_participants.is_empty() {
            ctx.ws_send(Event::Error(Error::InvalidParticipantTargets {
                participant_ids: invalid_participants,
            }));

            return Ok(false);
        }

        Ok(true)
    }

    async fn accept_whisper_invite(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        whisper_id: WhisperId,
    ) -> Result<(), SignalingModuleError> {
        let Some(whisper_group) = self.get_whisper_group(ctx, whisper_id).await? else {
            return Ok(());
        };

        if whisper_group.has_accepted(&self.participant_id) {
            ctx.ws_send(Event::Error(Error::AlreadyAccepted));
            return Ok(());
        }

        let token = self.create_access_token(whisper_id)?;

        ctx.volatile
            .storage()
            .update_participant_state(
                self.room_id,
                whisper_id,
                self.participant_id,
                WhisperState::Accepted,
            )
            .await?;

        ctx.ws_send(Event::WhisperToken(WhisperToken { whisper_id, token }));

        let message = exchange::Message::WhisperAccepted(WhisperAccepted {
            whisper_id,
            participant_id: self.participant_id,
        });

        self.notify_other_participants(ctx, whisper_group.participants.keys(), message);

        Ok(())
    }

    async fn destroy_whisper_room(&self, whisper_id: WhisperId) {
        if let Err(e) = self
            .params
            .room_client
            .delete_room(&whisper_id.to_string())
            .await
        {
            log::debug!(
                "failed to remove livekit whisper room for room {}: {}",
                self.room_id,
                e
            )
        }
    }

    /// Create Room and AccessToken
    ///
    /// Returns the access token
    async fn create_room_and_access_token(
        &self,
        whisper_id: WhisperId,
    ) -> Result<String, SignalingModuleError> {
        self.params
            .room_client
            .create_room(&whisper_id.to_string(), CreateRoomOptions::default())
            .await
            .whatever_context::<&str, SignalingModuleError>(
                "Failed to create livekit whisper room",
            )?;

        let token = self.create_access_token(whisper_id)?;

        Ok(token)
    }

    fn create_access_token(&self, whisper_id: WhisperId) -> Result<String, SignalingModuleError> {
        let identity = &self.participant_id.to_string();

        let access_token = AccessToken::with_api_key(
            &self.params.settings.api_key,
            &self.params.settings.api_secret,
        )
        .with_name(identity)
        .with_identity(identity)
        .with_grants(VideoGrants {
            room_create: false,
            room_list: false,
            room_record: false,
            room_admin: false,
            room_join: true,
            room: whisper_id.to_string(),
            can_publish: true,
            can_subscribe: true,
            can_publish_data: false,
            can_publish_sources: vec![TrackSource::Microphone.as_str_name().to_lowercase()],
            can_update_own_metadata: false,
            ingress_admin: false,
            hidden: false,
            recorder: false,
        })
        .with_ttl(ACCESS_TOKEN_TTL)
        .to_jwt()
        .whatever_context::<&str, SignalingModuleError>("Failed to create livekit access-token")?;

        Ok(access_token)
    }

    async fn leave_all_whisper_groups(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        let whisper_groups = ctx
            .volatile
            .storage()
            .get_all_whisper_group_ids(self.room_id)
            .await?;

        for whisper_id in whisper_groups {
            let participants = ctx
                .volatile
                .storage()
                .get_whisper_group(self.room_id, whisper_id)
                .await?;

            if participants
                .iter()
                .any(|(participant_id, _)| participant_id == &self.participant_id)
            {
                self.leave_whisper_group(
                    ctx,
                    WhisperGroup {
                        whisper_id,
                        participants,
                    },
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn cleanup_whisper_groups(&self, ctx: &mut DestroyContext<'_>, room_id: SignalingRoomId) {
        let whisper_ids = match ctx
            .volatile
            .storage()
            .get_all_whisper_group_ids(room_id)
            .await
        {
            Ok(whisper_ids) => whisper_ids,
            Err(e) => {
                log::error!(
                    "Failed to get all whisper groups for room {}: {}",
                    room_id,
                    e
                );
                return;
            }
        };

        for whisper_id in whisper_ids {
            if let Err(e) = ctx
                .volatile
                .storage()
                .delete_whisper_group(room_id, whisper_id)
                .await
            {
                log::error!(
                    "Failed to delete whisper group {} in room {}: {}",
                    whisper_id,
                    room_id,
                    e
                );
            }
            self.destroy_whisper_room(whisper_id).await;
        }
    }

    fn is_group_creator(&self, whisper_group: &WhisperGroup) -> bool {
        match whisper_group.participants.get(&self.participant_id) {
            Some(state) => state == &WhisperState::Creator,
            None => false,
        }
    }
}
