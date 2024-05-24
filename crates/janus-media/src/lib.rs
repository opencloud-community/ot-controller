// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! # Media Module
//!
//! ## Functionality
//!
//! Handles media related messages and manages their respective forwarding to janus-gateway via rabbitmq.
use std::{collections::BTreeSet, sync::Arc};

use mcu::{
    LinkDirection, McuPool, MediaSessionKey, PublishConfiguration, Request, Response,
    TrickleMessage, WebRtcEvent,
};
use opentalk_controller_settings::SharedSettings;
use opentalk_signaling_core::{
    control::{self, storage::ControlStorage as _},
    DestroyContext, Event, InitContext, ModuleContext, SignalingModule, SignalingModuleError,
    SignalingModuleInitData, SignalingRoomId, VolatileStorageBackend,
};
use opentalk_types::{
    core::ParticipantId,
    signaling::{
        media::{
            command::{self, MediaCommand, Target, TargetConfigure, TargetSubscribe},
            event::{self, Error, Link, MediaEvent, MediaStatus, Sdp, SdpCandidate, Source},
            peer_state::MediaPeerState,
            state::MediaState,
            MediaSessionState, MediaSessionType, ParticipantMediaState, ParticipantSpeakingState,
            SpeakingState, TrickleCandidate, UpdateSpeakingState, NAMESPACE,
        },
        Role,
    },
};
use sessions::MediaSessions;
use snafu::{whatever, OptionExt, Report};
use storage::MediaStorage as _;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

mod exchange;
mod mcu;
mod sessions;
mod settings;
mod storage;

pub struct Media {
    id: ParticipantId,
    room: SignalingRoomId,

    mcu: Arc<McuPool>,
    media: MediaSessions,

    state: ParticipantMediaState,
}

fn process_metrics_for_media_session_state(
    ctx: &ModuleContext<'_, Media>,
    session_type: &MediaSessionType,
    previous: &Option<MediaSessionState>,
    new: &MediaSessionState,
) {
    if let Some(metrics) = ctx.metrics() {
        let previous = previous.unwrap_or(MediaSessionState {
            video: false,
            audio: false,
        });

        if !previous.audio && new.audio {
            metrics.increment_participants_with_audio_count(session_type.as_str());
        } else if previous.audio && !new.audio {
            metrics.decrement_participants_with_audio_count(session_type.as_str());
        }

        if !previous.video && new.video {
            metrics.increment_participants_with_video_count(session_type.as_str());
        } else if previous.video && !new.video {
            metrics.decrement_participants_with_video_count(session_type.as_str());
        }
    }
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
    fn storage_ref(&self) -> &dyn storage::MediaStorage {
        if self.storage.is_left() {
            self.storage.as_ref().left().unwrap()
        } else {
            self.storage.as_ref().right().unwrap()
        }
    }

    fn storage_mut(&mut self) -> &mut dyn storage::MediaStorage {
        if self.storage.is_left() {
            self.storage.as_mut().left().unwrap()
        } else {
            self.storage.as_mut().right().unwrap()
        }
    }
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for Media {
    const NAMESPACE: &'static str = NAMESPACE;

    type Params = Arc<McuPool>;

    type Incoming = MediaCommand;
    type Outgoing = MediaEvent;
    type ExchangeMessage = exchange::Message;

    type ExtEvent = (MediaSessionKey, WebRtcEvent);

    type FrontendData = MediaState;
    type PeerFrontendData = MediaPeerState;

    type Volatile = VolatileWrapper;

    async fn init(
        mut ctx: InitContext<'_, Self>,
        mcu: &Self::Params,
        _protocol: &'static str,
    ) -> Result<Option<Self>, SignalingModuleError> {
        let (media_sender, janus_events) = mpsc::channel(12);

        let state = ParticipantMediaState::default();

        let id = ctx.participant_id();
        let room = ctx.room_id();

        ctx.redis_conn().set_media_state(room, id, &state).await?;
        ctx.add_event_stream(ReceiverStream::new(janus_events));

        if !screen_share_requires_permission(&mcu.shared_settings) {
            ctx.redis_conn().add_presenter(room, id).await?;
        }

        Ok(Some(Self {
            id,
            room,
            mcu: mcu.clone(),
            media: MediaSessions::new(ctx.participant_id(), media_sender),
            state,
        }))
    }

    async fn on_event(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        event: Event<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        match event {
            Event::WsMessage(MediaCommand::PublishComplete(info)) => {
                let previous_session_state = self.state.get(info.media_session_type);

                process_metrics_for_media_session_state(
                    &ctx,
                    &info.media_session_type,
                    &previous_session_state,
                    &info.media_session_state,
                );

                let old_state = self
                    .state
                    .insert(info.media_session_type, info.media_session_state);

                ctx.redis_conn()
                    .set_media_state(self.room, self.id, &self.state)
                    .await?;

                ctx.invalidate_data();

                if Some(info.media_session_state) != old_state {
                    self.handle_publish_state(info.media_session_type, info.media_session_state)
                        .await?;
                }
            }
            Event::WsMessage(MediaCommand::UpdateMediaSession(info)) => {
                if info.media_session_type == MediaSessionType::Screen
                    && ctx.role() != Role::Moderator
                    && !ctx.redis_conn().is_presenter(self.room, self.id).await?
                {
                    ctx.ws_send(Error::PermissionDenied);
                    return Ok(());
                }

                let previous_session_state = self.state.get(info.media_session_type);

                process_metrics_for_media_session_state(
                    &ctx,
                    &info.media_session_type,
                    &previous_session_state,
                    &info.media_session_state,
                );

                if let Some(state) = self.state.get_mut(info.media_session_type) {
                    let old_state = *state;
                    *state = info.media_session_state;

                    ctx.redis_conn()
                        .set_media_state(self.room, self.id, &self.state)
                        .await?;

                    ctx.invalidate_data();

                    if info.media_session_state != old_state {
                        self.handle_publish_state(
                            info.media_session_type,
                            info.media_session_state,
                        )
                        .await?;

                        if old_state.audio && !info.media_session_state.audio {
                            let timestamp = ctx.timestamp();
                            let is_speaking = false;
                            ctx.redis_conn()
                                .set_speaking_state(self.room, self.id, is_speaking, timestamp)
                                .await?;
                            ctx.exchange_publish(
                                control::exchange::current_room_all_participants(self.room),
                                exchange::Message::SpeakerStateUpdated(ParticipantSpeakingState {
                                    participant: self.id,
                                    speaker: SpeakingState {
                                        is_speaking,
                                        updated_at: timestamp,
                                    },
                                }),
                            );
                        }
                    }
                }
            }
            Event::WsMessage(MediaCommand::ModeratorMute(moderator_mute)) => {
                self.handle_moderator_mute(&mut ctx, moderator_mute).await?;
            }
            Event::WsMessage(MediaCommand::Unpublish(assoc)) => {
                self.media.remove_publisher(assoc.media_session_type).await;
                let previous_session_state = self.state.remove(assoc.media_session_type);

                process_metrics_for_media_session_state(
                    &ctx,
                    &assoc.media_session_type,
                    &previous_session_state,
                    &MediaSessionState {
                        audio: false,
                        video: false,
                    },
                );

                ctx.redis_conn()
                    .set_media_state(self.room, self.id, &self.state)
                    .await?;

                ctx.invalidate_data();
            }
            Event::WsMessage(MediaCommand::Publish(targeted)) => {
                if targeted.target.media_session_type == MediaSessionType::Screen
                    && ctx.role() != Role::Moderator
                    && !ctx.redis_conn().is_presenter(self.room, self.id).await?
                {
                    ctx.ws_send(Error::PermissionDenied);

                    return Ok(());
                }

                if let Err(e) = self
                    .handle_sdp_offer(
                        &mut ctx,
                        targeted.target.target,
                        targeted.target.media_session_type,
                        targeted.sdp,
                    )
                    .await
                {
                    log::error!(
                        "Failed to handle sdp offer for {:?}, {:?}",
                        targeted.target,
                        e
                    );
                    ctx.ws_send(Error::InvalidSdpOffer);
                }
            }
            Event::WsMessage(MediaCommand::SdpAnswer(targeted)) => {
                if let Err(e) = self
                    .handle_sdp_answer(
                        targeted.target.target,
                        targeted.target.media_session_type,
                        targeted.sdp,
                    )
                    .await
                {
                    log::error!(
                        "Failed to handle sdp answer {:?}, {}",
                        targeted.target,
                        Report::from_error(e)
                    );
                    ctx.ws_send(Error::HandleSdpAnswer);
                }
            }
            Event::WsMessage(MediaCommand::SdpCandidate(targeted)) => {
                if let Err(e) = self
                    .handle_sdp_candidate(
                        targeted.target.target,
                        targeted.target.media_session_type,
                        targeted.candidate,
                    )
                    .await
                {
                    log::error!(
                        "Failed to handle sdp candidate {:?}, {:?}",
                        targeted.target,
                        e
                    );
                    ctx.ws_send(Error::InvalidCandidate);
                }
            }
            Event::WsMessage(MediaCommand::SdpEndOfCandidates(target)) => {
                if let Err(e) = self
                    .handle_sdp_end_of_candidates(target.target, target.media_session_type)
                    .await
                {
                    log::error!(
                        "Failed to handle sdp end-of-candidates {:?}, {:?}",
                        target,
                        e
                    );
                    ctx.ws_send(Error::InvalidEndOfCandidates);
                }
            }
            Event::WsMessage(MediaCommand::Subscribe(subscribe)) => {
                // Check that the subscription target is inside the room
                if !ctx
                    .redis_conn()
                    .participants_contains(self.room, subscribe.target.target)
                    .await?
                {
                    // just discard, shouldn't happen often
                    return Ok(());
                }

                if let Err(e) = self.handle_sdp_request_offer(&mut ctx, subscribe).await {
                    log::error!(
                        "Failed to handle sdp request-offer {:?}, {:?}",
                        subscribe,
                        e
                    );
                    ctx.ws_send(Error::InvalidRequestOffer(subscribe.target.into()));
                }
            }
            Event::WsMessage(MediaCommand::Resubscribe(target)) => {
                if let Err(e) = self.handle_sdp_re_request_offer(&mut ctx, target).await {
                    log::error!(
                        "Failed to handle resubscribe {:?}, {}",
                        target,
                        Report::from_error(e)
                    );
                    ctx.ws_send(Error::InvalidRequestOffer(target.into()));
                }
            }
            Event::WsMessage(MediaCommand::Configure(configure)) => {
                let target = configure.target;
                if let Err(e) = self.handle_configure(configure).await {
                    log::error!(
                        "Failed to handle configure request {}",
                        Report::from_error(e)
                    );
                    ctx.ws_send(Error::InvalidConfigureRequest(target.into()));
                }
            }

            Event::WsMessage(MediaCommand::GrantPresenterRole(selection)) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::PermissionDenied);

                    return Ok(());
                }

                ctx.exchange_publish(
                    control::exchange::current_room_all_participants(self.room),
                    exchange::Message::PresenterGranted(selection),
                )
            }
            Event::WsMessage(MediaCommand::RevokePresenterRole(selection)) => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::PermissionDenied);

                    return Ok(());
                }

                ctx.exchange_publish(
                    control::exchange::current_room_all_participants(self.room),
                    exchange::Message::PresenterRevoked(selection),
                )
            }

            Event::WsMessage(MediaCommand::UpdateSpeakingState(UpdateSpeakingState {
                is_speaking,
            })) => {
                let timestamp = ctx.timestamp();
                ctx.redis_conn()
                    .set_speaking_state(self.room, self.id, is_speaking, timestamp)
                    .await?;
                ctx.exchange_publish(
                    control::exchange::current_room_all_participants(self.room),
                    exchange::Message::SpeakerStateUpdated(ParticipantSpeakingState {
                        participant: self.id,
                        speaker: SpeakingState {
                            is_speaking,
                            updated_at: timestamp,
                        },
                    }),
                );
            }

            Event::Ext((media_session_key, message)) => match message {
                WebRtcEvent::AssociatedMcuDied => {
                    self.remove_broken_media_session(&mut ctx, media_session_key)
                        .await?;
                    ctx.ws_send(MediaEvent::WebrtcDown(media_session_key.into()))
                }
                WebRtcEvent::WebRtcUp => {
                    ctx.ws_send(MediaEvent::WebrtcUp(media_session_key.into()))
                }
                WebRtcEvent::Media(media) => ctx.ws_send(MediaEvent::MediaStatus(MediaStatus {
                    source: media_session_key.into(),
                    kind: media.kind,
                    receiving: media.receiving,
                })),
                WebRtcEvent::WebRtcDown => {
                    ctx.ws_send(MediaEvent::WebrtcDown(media_session_key.into()));

                    self.gracefully_remove_media_session(&mut ctx, media_session_key)
                        .await?;
                }
                WebRtcEvent::SlowLink(link_direction) => {
                    let direction = match link_direction {
                        LinkDirection::Upstream => event::LinkDirection::Upstream,
                        LinkDirection::Downstream => event::LinkDirection::Downstream,
                    };

                    ctx.ws_send(MediaEvent::WebrtcSlow(Link {
                        direction,
                        source: media_session_key.into(),
                    }))
                }
                WebRtcEvent::Trickle(trickle_msg) => match trickle_msg {
                    // This send by Janus when in full-trickle mode.
                    TrickleMessage::Completed => {
                        ctx.ws_send(MediaEvent::SdpEndOfCandidates(media_session_key.into()));
                    }
                    TrickleMessage::Candidate(opentalk_janus_client::TrickleCandidate {
                        sdp_m_line_index,
                        candidate,
                    }) => {
                        ctx.ws_send(MediaEvent::SdpCandidate(SdpCandidate {
                            candidate: TrickleCandidate {
                                sdp_m_line_index,
                                candidate,
                            },
                            source: media_session_key.into(),
                        }));
                    }
                },
            },
            Event::Exchange(exchange::Message::RequestMute(request_mute)) => {
                ctx.ws_send(request_mute);
            }
            Event::Exchange(exchange::Message::PresenterGranted(selection)) => {
                if !selection.participant_ids.contains(&self.id) {
                    return Ok(());
                }

                if ctx.redis_conn().is_presenter(self.room, self.id).await? {
                    // already presenter
                    return Ok(());
                }

                ctx.redis_conn().add_presenter(self.room, self.id).await?;

                ctx.ws_send(MediaEvent::PresenterGranted);

                ctx.invalidate_data();
            }
            Event::Exchange(exchange::Message::PresenterRevoked(selection)) => {
                if !selection.participant_ids.contains(&self.id) {
                    return Ok(());
                }

                if !ctx.redis_conn().is_presenter(self.room, self.id).await? {
                    // already not a presenter
                    return Ok(());
                }

                ctx.redis_conn()
                    .remove_presenter(self.room, self.id)
                    .await?;

                // terminate screen share
                if self.state.get(MediaSessionType::Screen).is_some()
                    && ctx.role() != Role::Moderator
                {
                    self.media.remove_publisher(MediaSessionType::Screen).await;
                    self.state.remove(MediaSessionType::Screen);

                    ctx.redis_conn()
                        .set_media_state(self.room, self.id, &self.state)
                        .await?;
                }

                ctx.ws_send(MediaEvent::PresenterRevoked);

                ctx.invalidate_data();
            }
            Event::Exchange(exchange::Message::SpeakerStateUpdated(participant_speaker)) => {
                ctx.ws_send(MediaEvent::SpeakerUpdated(participant_speaker));
            }

            Event::ParticipantJoined(id, evt_state) => {
                let state = ctx
                    .redis_conn()
                    .get_media_state(self.room, id)
                    .await?
                    .unwrap_or_default();

                let is_presenter = ctx.redis_conn().is_presenter(self.room, id).await?;

                *evt_state = Some(MediaPeerState {
                    state,
                    is_presenter,
                })
            }
            Event::ParticipantUpdated(id, evt_state) => {
                let state = ctx.redis_conn().get_media_state(self.room, id).await?;

                if let Some(state) = &state {
                    self.media.remove_dangling_subscriber(id, state).await;
                }

                let is_presenter = ctx.redis_conn().is_presenter(self.room, id).await?;

                *evt_state = Some(MediaPeerState {
                    state: state.unwrap_or_default(),
                    is_presenter,
                });
            }
            Event::ParticipantLeft(id) => {
                self.media.remove_subscribers(id).await;
            }
            Event::Joined {
                control_data: _,
                frontend_data,
                participants,
            } => {
                let participant_ids = Vec::from_iter(participants.keys().cloned());
                for (&id, evt_state) in participants {
                    let state = ctx
                        .redis_conn()
                        .get_media_state(self.room, id)
                        .await?
                        .unwrap_or_default();

                    let is_presenter = ctx.redis_conn().is_presenter(self.room, id).await?;

                    *evt_state = Some(MediaPeerState {
                        state,
                        is_presenter,
                    })
                }

                let is_presenter = ctx.redis_conn().is_presenter(self.room, self.id).await?;
                let speakers = ctx
                    .redis_conn()
                    .get_speaking_state_multiple_participants(self.room, &participant_ids)
                    .await?;

                *frontend_data = Some(MediaState {
                    is_presenter,
                    speakers,
                })
            }
            Event::Leaving => {
                if let Err(e) = ctx
                    .redis_conn()
                    .delete_media_state(self.room, self.id)
                    .await
                {
                    log::error!(
                        "Media module for {} failed to remove its state data from redis, {}",
                        self.id,
                        e
                    );
                }

                // Spawn destroying all the handles as it doesn't need to be synchronized
                // and should not block the leaving process
                tokio::task::spawn_local(self.media.destroy());
            }
            Event::RaiseHand | Event::LowerHand { .. } | Event::RoleUpdated(_) => {}
        }

        Ok(())
    }

    async fn on_destroy(self, mut ctx: DestroyContext<'_>) {
        if ctx.destroy_room() {
            if let Err(e) = ctx.redis_conn().clear_presenters(self.room).await {
                log::error!(
                    "Media module for failed to remove presenter key on room destoy, {}",
                    e
                );
            }

            let participants = ctx
                .redis_conn()
                .get_all_participants(self.room)
                .await
                .unwrap_or_else(|e| {
                    log::error!(
                        "Failed to load room participants, {}",
                        Report::from_error(e)
                    );
                    BTreeSet::new()
                });

            if let Err(e) = ctx
                .redis_conn()
                .delete_speaking_state_multiple_participants(
                    self.room,
                    &Vec::from_iter(participants),
                )
                .await
            {
                log::error!(
                    "Media module for failed to remove speakers on room destoy, {}",
                    e
                );
            }
        }
    }

    async fn build_params(
        data: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
        let mcu_pool = McuPool::build(
            data.startup_settings.as_ref(),
            data.shared_settings.clone(),
            data.rabbitmq_pool.clone(),
            data.volatile.clone().into(),
            data.shutdown.subscribe(),
            data.reload.subscribe(),
        )
        .await?;

        Ok(Some(mcu_pool))
    }
}

impl Media {
    /// Send mute requests to the targeted participants
    ///
    /// Fails if the issuing user is not a moderator.
    async fn handle_moderator_mute(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        moderator_mute: command::RequestMute,
    ) -> Result<(), SignalingModuleError> {
        if ctx.role() != Role::Moderator {
            ctx.ws_send(Error::PermissionDenied);

            return Ok(());
        }

        let room_participants = ctx.redis_conn().get_all_participants(self.room).await?;

        let request_mute = event::RequestMute {
            issuer: self.id,
            force: moderator_mute.force,
        };

        for target in moderator_mute.targets {
            if !room_participants.contains(&target) {
                continue;
            }

            ctx.exchange_publish(
                control::exchange::current_room_by_participant_id(self.room, target),
                exchange::Message::RequestMute(request_mute.clone()),
            )
        }

        Ok(())
    }

    /// Gracefully removes the media session that is associated with the provided MediaSessionKey
    ///
    /// Send detach and destroy messages to janus in order to remove a media session gracefully.
    #[tracing::instrument(level = "debug", skip(self, ctx))]
    async fn gracefully_remove_media_session(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        media_session_key: MediaSessionKey,
    ) -> Result<(), SignalingModuleError> {
        if media_session_key.0 == self.id {
            log::trace!("Removing publisher {}", media_session_key);
            self.media.remove_publisher(media_session_key.1).await;
            self.state.remove(media_session_key.1);

            ctx.redis_conn()
                .set_media_state(self.room, self.id, &self.state)
                .await?;

            ctx.invalidate_data();
        } else {
            log::trace!("Removing subscriber {}", media_session_key);
            self.media.remove_subscriber(&media_session_key).await;
        }
        Ok(())
    }

    /// Kills a media session
    ///
    /// Opposed to [`Media::gracefully_remove_media_session`], this function will not inform janus
    /// about any changes to the media session.
    #[tracing::instrument(level = "debug", skip(self, ctx))]
    async fn remove_broken_media_session(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        media_session_key: MediaSessionKey,
    ) -> Result<(), SignalingModuleError> {
        if media_session_key.0 == self.id {
            log::trace!("Removing broken publisher {}", media_session_key);
            self.media
                .remove_broken_publisher(media_session_key.1)
                .await;
            self.state.remove(media_session_key.1);

            ctx.redis_conn()
                .set_media_state(self.room, self.id, &self.state)
                .await?;

            ctx.invalidate_data();
        } else {
            log::trace!("Removing broken subscriber {}", media_session_key);
            self.media
                .remove_broken_subscriber(&media_session_key)
                .await;
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, ctx, offer))]
    async fn handle_sdp_offer(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        target: ParticipantId,
        media_session_type: MediaSessionType,
        offer: String,
    ) -> Result<(), SignalingModuleError> {
        if target == self.id {
            // Get the publisher and create if it doesn't exists
            let publisher = if let Some(publisher) = self.media.get_publisher(media_session_type) {
                publisher
            } else {
                self.media
                    .create_publisher(&self.mcu, media_session_type)
                    .await?
            };

            // Send to offer and await the result
            let response = publisher.send_message(Request::SdpOffer(offer)).await?;

            match response {
                Response::SdpAnswer(answer) => {
                    ctx.ws_send(MediaEvent::SdpAnswer(Sdp {
                        sdp: answer.sdp(),
                        source: Source {
                            source: target,
                            media_session_type,
                        },
                    }));
                }
                Response::SdpOffer(_) | Response::None => {
                    whatever!("Expected McuResponse::SdpAnswer(..), got {:?}", response)
                }
            }

            Ok(())
        } else {
            whatever!("Invalid target id, cannot send offer to other participants")
        }
    }

    #[tracing::instrument(level = "debug", skip(self, answer))]
    async fn handle_sdp_answer(
        &mut self,
        target: ParticipantId,
        media_session_type: MediaSessionType,
        answer: String,
    ) -> Result<(), SignalingModuleError> {
        if target == self.id {
            // Get the publisher and create if it doesn't exists
            let publisher = self
                .media
                .get_publisher(media_session_type)
                .whatever_context::<&str, SignalingModuleError>(
                    "SDP Answer for nonexistent publisher received",
                )?;

            // Send to offer and await the result
            publisher.send_message(Request::SdpAnswer(answer)).await?;
        } else {
            let subscriber = self
                .media
                .get_subscriber(target, media_session_type)
                .whatever_context::<&str, SignalingModuleError>(
                "SDP Answer for nonexisting subscriber received",
            )?;

            subscriber.send_message(Request::SdpAnswer(answer)).await?;
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, candidate))]
    async fn handle_sdp_candidate(
        &mut self,
        target: ParticipantId,
        media_session_type: MediaSessionType,
        TrickleCandidate {
            sdp_m_line_index,
            candidate,
        }: TrickleCandidate,
    ) -> Result<(), SignalingModuleError> {
        let req = Request::Candidate(opentalk_janus_client::TrickleCandidate {
            sdp_m_line_index,
            candidate,
        });

        if target == self.id {
            let publisher = self
                .media
                .get_publisher(media_session_type)
                .whatever_context::<&str, SignalingModuleError>(
                    "SDP candidate for nonexistent publisher received",
                )?;

            publisher.send_message(req).await?;
        } else {
            let subscriber = self
                .media
                .get_subscriber(target, media_session_type)
                .whatever_context::<&str, SignalingModuleError>(
                "SDP candidate for nonexisting subscriber received",
            )?;

            subscriber.send_message(req).await?;
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn handle_sdp_end_of_candidates(
        &mut self,
        target: ParticipantId,
        media_session_type: MediaSessionType,
    ) -> Result<(), SignalingModuleError> {
        if target == self.id {
            let publisher = self
                .media
                .get_publisher(media_session_type)
                .whatever_context::<&str, SignalingModuleError>(
                    "SDP end-of-candidates for nonexistent publisher received",
                )?;

            publisher.send_message(Request::EndOfCandidates).await?;
        } else {
            let subscriber = self
                .media
                .get_subscriber(target, media_session_type)
                .whatever_context::<&str, SignalingModuleError>(
                "SDP end-of-candidates for nonexisting subscriber received",
            )?;

            subscriber.send_message(Request::EndOfCandidates).await?;
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, ctx))]
    async fn handle_sdp_request_offer(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        subscribe: TargetSubscribe,
    ) -> Result<(), SignalingModuleError> {
        let target = subscribe.target.target;
        let media_session_type = subscribe.target.media_session_type;

        if self.id == subscribe.target.target {
            // Usually subscribing to self should be possible but cannot be realized with the
            // current messaging model. The frontend wouldn't know if a sdp-offer is an update
            // to the publish or a response to the requestOffer (subscribe)
            whatever!("Cannot request offer for self");
        }

        let subscriber =
            if let Some(subscriber) = self.media.get_subscriber(target, media_session_type) {
                subscriber
            } else {
                self.media
                    .create_subscriber(self.mcu.as_ref(), target, media_session_type)
                    .await?
            };

        let response = subscriber
            .send_message(Request::RequestOffer {
                without_video: subscribe.without_video,
            })
            .await?;

        match response {
            Response::SdpOffer(offer) => {
                ctx.ws_send(MediaEvent::SdpOffer(Sdp {
                    sdp: offer.sdp(),
                    source: Source {
                        source: target,
                        media_session_type,
                    },
                }));
            }
            Response::SdpAnswer(_) | Response::None => {
                whatever!("Expected McuResponse::SdpOffer(..) got {:?}", response)
            }
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, ctx))]
    async fn handle_sdp_re_request_offer(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        target: Target,
    ) -> Result<(), SignalingModuleError> {
        let media_session_type = target.media_session_type;
        let target = target.target;

        if self.id == target {
            whatever!("Cannot re-request offer for self");
        }

        let subscriber =
            self.media
                .get_subscriber(target, media_session_type)
                .whatever_context::<&str, SignalingModuleError>("No subscriber for target found")?;

        let sdp_offer = subscriber.restart().await?;

        ctx.ws_send(MediaEvent::SdpOffer(Sdp {
            sdp: sdp_offer,
            source: Source {
                source: target,
                media_session_type,
            },
        }));

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn handle_publish_state(
        &mut self,
        media_session_type: MediaSessionType,
        state: MediaSessionState,
    ) -> Result<(), SignalingModuleError> {
        if let Some(publisher) = self.media.get_publisher(media_session_type) {
            publisher
                .send_message(Request::PublisherConfigure(PublishConfiguration {
                    video: state.video,
                    audio: state.audio,
                }))
                .await?;
        } else {
            log::info!("Attempt to configure none existing publisher");
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn handle_configure(
        &mut self,
        configure: TargetConfigure,
    ) -> Result<(), SignalingModuleError> {
        if let Some(subscriber) = self
            .media
            .get_subscriber(configure.target.target, configure.target.media_session_type)
        {
            subscriber
                .send_message(Request::SubscriberConfigure(configure.configuration))
                .await?;
        } else {
            log::info!("Attempt to configure none existing subscriber");
        }

        Ok(())
    }
}

pub fn screen_share_requires_permission(shared_settings: &SharedSettings) -> bool {
    shared_settings
        .load()
        .defaults
        .screen_share_requires_permission
}

/// Check for deprecated settings, and print warnings if any are found.
pub fn check_for_deprecated_settings(
    settings: &opentalk_controller_settings::Settings,
) -> Result<Vec<&'static str>, SignalingModuleError> {
    let mcu_config = settings::JanusMcuConfig::extract(settings)?;

    let mut found = Vec::new();
    if mcu_config.speaker_focus_packets.is_some() {
        found.push("room_server.speaker_focus_packets");
    }
    if mcu_config.speaker_focus_level.is_some() {
        found.push("room_server.speaker_focus_level");
    }

    Ok(found)
}
