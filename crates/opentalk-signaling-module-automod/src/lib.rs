// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! # Auto-Moderation Module
//!
//! ## Functionality
//!
//! On room startup the automod is disabled.
//!
//! Selecting the options for the automod is managed by the frontend and this module does not
//! provide templates or anything else
//!
//! Unlike other modules the automod has commands with different levels of required permissions.
//! These permissions are not yet defined, thus only the room owner is moderator.
//!
//! Following selection_strategies are defined:
//!
//! - `None`: No automatic reselection happens after the current speaker yields. The next one must
//!     always be selected by the moderator. The moderator may choose a participant directly
//!     or let the controller choose one randomly. For that the controller holds a `allow_list`
//!     which is a set of participants which are able to be randomly selected. Furthermore the
//!     controller will hold a list of start/stop speaker events. That list can be used to avoid
//!     double selections (option) when randomly choosing a participant.
//!
//! - `Playlist`: The playlist-strategy requires a playlist of participants. This list will be
//!     stored ordered inside the controller. Whenever a speaker yields the controller will
//!     automatically choose the next participant in the list to be the next speaker.
//!
//!     A moderator may choose to skip over a speaker. That can be done by selecting the next one or
//!     let the controller choose someone random from the playlist.
//!     The playlist can, while the automod is active, be edited.
//!
//! - `Random`: This strategy behaves like `None` but will always choose the next speaker
//!     randomly from the `allow_list` as soon as the current speaker yields.
//!
//! - `Nomination`: This strategy behaves like `None` but requires the current speaker to nominate
//!     the next participant to be speaker. The nominated participant MUST be inside the
//!     `allow_list` and if double selection is not enabled the controller will check if the
//!     nominated participant already was a speaker.
//!
//! ### Lifecycle
//!
//! As soon if a moderator starts the automod, the automod-module of that
//! participant will set the config inside the storage and then send a start message to all other
//! participants.
//!
//! To avoid multiple concurrent actions the module will acquire a redlock to signal the
//! ownership of the automod while doing the work.
//!
//! Receiving the start-message will not change the state of the automod module. Instead it reads
//! out the config from the message and forwards it to the frontend after removing the list of
//! participants if the parameters requires it.
//!
//! The selection of the first speaker must be done by the frontend then, depending of the
//! `selection_strategy`, will the automod continue running until finished or stopped.
//!
//! Once the active speaker yields or it's time runs out, its automod module is responsible to
//! select the next speaker (if the `selection_strategy` requires it). This behavior MUST only
//! be executed after ensuring that this participant is in fact still the speaker.
//!
//! If the participant leaves while being speaker, its automod-module must execute the same
//! behavior as if the participants simply yielded without selecting the next one (which would be
//! required for the `nominate` `selection_strategy`. A moderator has to intervene in this
//! situation).
//!
//! Moderators will always be able to execute a re-selection of the current speaker regardless of
//! the `selection_strategy`.

mod exchange;
mod state_machine;
mod storage;

use std::time::Duration;

use either::Either;
use futures::{stream::once, FutureExt};
use opentalk_signaling_core::{
    control, DestroyContext, Event, InitContext, ModuleContext, RoomLockingProvider,
    SignalingModule, SignalingModuleError, SignalingModuleInitData, SignalingRoomId,
    VolatileStorage,
};
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::{ParticipantId, Role};
use opentalk_types_signaling_automod::{
    self,
    command::{AutomodCommand, Select, Start, Yield},
    config::{FrontendConfig, Parameter, SelectionStrategy},
    event::{AutomodEvent, Error, RemainingUpdated, SpeakerUpdated, StartAnimation, StoppedReason},
    state::AutomodState,
    MODULE_ID,
};
use snafu::{whatever, ResultExt};
use state_machine::StateMachineOutput;
use storage::AutomodStorage;
use tokio::time::sleep;
use uuid::Uuid;

use crate::{exchange::Message, storage::StorageConfig};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ExpiryId(Uuid);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AnimationId(Uuid);

pub enum TimerEvent {
    AnimationEnd(AnimationId, ParticipantId),
    Expiry(ExpiryId),
}

pub struct Automod {
    id: ParticipantId,
    room: SignalingRoomId,

    current_expiry_id: Option<ExpiryId>,
    current_animation_id: Option<AnimationId>,
}

trait AutomodStorageProvider {
    fn storage(&mut self) -> &mut dyn AutomodStorage;
}

impl AutomodStorageProvider for VolatileStorage {
    fn storage(&mut self) -> &mut dyn AutomodStorage {
        match self.as_mut() {
            Either::Left(v) => v,
            Either::Right(v) => v,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for Automod {
    const NAMESPACE: ModuleId = MODULE_ID;

    type Params = ();

    type Incoming = AutomodCommand;
    type Outgoing = AutomodEvent;
    type ExchangeMessage = Message;

    type ExtEvent = TimerEvent;

    type FrontendData = AutomodState;
    type PeerFrontendData = ();

    async fn init(
        ctx: InitContext<'_, Self>,
        _params: &Self::Params,
        _protocol: &'static str,
    ) -> Result<Option<Self>, SignalingModuleError> {
        Ok(Some(Self {
            id: ctx.participant_id(),
            room: ctx.room_id(),
            current_expiry_id: None,
            current_animation_id: None,
        }))
    }

    async fn on_event(
        &mut self,
        ctx: ModuleContext<'_, Self>,
        event: Event<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        match event {
            Event::Joined {
                control_data: _,
                frontend_data,
                participants: _,
            } => self.on_joined(ctx, frontend_data).await,
            Event::Leaving => self.on_leaving(ctx).await,
            Event::ParticipantJoined(_, _)
            | Event::ParticipantLeft(_)
            | Event::ParticipantUpdated(_, _)
            | Event::RoleUpdated(_) => {
                // ignored
                Ok(())
            }
            Event::WsMessage(msg) => self.on_ws_message(ctx, msg).await,
            Event::Exchange(msg) => self.on_exchange_msg(ctx, msg).await,
            Event::Ext(TimerEvent::AnimationEnd(animation_id, selection)) => {
                if let Some(current_animation_id) = self.current_animation_id {
                    if current_animation_id == animation_id {
                        self.on_animation_end(ctx, selection).await?;
                    }
                }

                Ok(())
            }
            Event::Ext(TimerEvent::Expiry(expiry_id)) => {
                if let Some(current_expiry_id) = self.current_expiry_id {
                    if current_expiry_id == expiry_id {
                        self.on_expired_event(ctx).await?;
                    }
                }

                Ok(())
            }
            Event::RaiseHand => {
                // TODO Add self to playlist if allowed
                Ok(())
            }
            Event::LowerHand => {
                // TODO
                Ok(())
            }
        }
    }

    async fn on_destroy(self, ctx: DestroyContext<'_>) {
        if ctx.destroy_room() {
            let storage = ctx.volatile.storage();
            let _ = storage.config_delete(self.room).await;
            let _ = storage.allow_list_delete(self.room).await;
            let _ = storage.playlist_delete(self.room).await;
            let _ = storage.history_delete(self.room).await;
        }
    }

    async fn build_params(
        _init: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
        Ok(Some(()))
    }
}

/// Macro to try an operation and if it fails unlock the guard and return an error
macro_rules! try_or_unlock {
    ($expr:expr; $ctx:ident, $guard:ident) => {
        match $expr {
            Ok(item) => item,
            Err(e) => {
                let _ = $ctx.volatile.room_locking().unlock_room($guard).await;
                whatever!("{e}")
            }
        }
    };
}

impl Automod {
    /// Called when participant joins a room.
    ///
    /// Checks if the automod is active by reading the config. If active set the `frontend_data`
    /// to the current config and automod state (history + remaining).
    #[tracing::instrument(name = "automod_on_joined", skip(self, ctx, frontend_data))]
    async fn on_joined(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        frontend_data: &mut Option<AutomodState>,
    ) -> Result<(), SignalingModuleError> {
        let guard = ctx.volatile.room_locking().lock_room(self.room).await?;

        let result = self.on_joined_inner(&mut ctx).await;

        ctx.volatile.room_locking().unlock_room(guard).await?;

        *frontend_data = result?;
        Ok(())
    }

    async fn on_joined_inner(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<Option<AutomodState>, SignalingModuleError> {
        let storage = ctx.volatile.storage();
        let config = storage.config_get(self.room).await?;

        // If config is some, automod is active and running
        let Some(config) = config else {
            // automod not active, just return nothing
            return Ok(None);
        };

        let speaker = storage.speaker_get(self.room).await?;
        let history = storage.history_get(self.room, config.started).await?;
        let auto_append = config.parameter.auto_append_on_join && !history.contains(&self.id);

        if auto_append {
            self.add_to_remaining(storage, config.parameter.selection_strategy)
                .await?;
        }

        let remaining = self
            .get_remaining(storage, config.parameter.selection_strategy)
            .await?;

        if auto_append {
            ctx.exchange_publish(
                control::exchange::current_room_all_participants(self.room),
                exchange::Message::RemainingUpdate(exchange::RemainingUpdate {
                    remaining: remaining.clone(),
                }),
            );
        }

        Ok(Some(AutomodState {
            config: FrontendConfig {
                parameter: config.parameter,
                history,
                remaining,
                issued_by: config.issued_by,
            }
            .into_public(),
            speaker,
        }))
    }

    /// Called right before a participants leaves.
    ///
    /// Removes this participants from the playlist, allow_list and sets the speaker to none if
    /// speaker is the leaving participant.
    #[tracing::instrument(name = "automod_on_leaving", skip(self, ctx))]
    async fn on_leaving(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        let guard = ctx.volatile.room_locking().lock_room(self.room).await?;

        let result = self.on_leaving_inner(&mut ctx).await;
        ctx.volatile.room_locking().unlock_room(guard).await?;
        result
    }

    async fn on_leaving_inner(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        let config = ctx.volatile.storage().config_get(self.room).await?;

        if let Some(config) = config {
            let mut removed = 0;
            removed += ctx
                .volatile
                .storage()
                .playlist_remove_all_occurrences(self.room, self.id)
                .await?;
            removed += ctx
                .volatile
                .storage()
                .allow_list_remove(self.room, self.id)
                .await?;

            if removed > 0 {
                let remaining = self
                    .get_remaining(ctx.volatile.storage(), config.parameter.selection_strategy)
                    .await?;
                ctx.exchange_publish(
                    control::exchange::current_room_all_participants(self.room),
                    exchange::Message::RemainingUpdate(exchange::RemainingUpdate { remaining }),
                );
            }

            let speaker = ctx.volatile.storage().speaker_get(self.room).await?;

            if speaker == Some(self.id) {
                self.select_next(ctx, config, None).await?;
            }
        }
        Ok(())
    }

    /// Called when the speaking time of the participant ends.
    ///
    /// Checks if automod is still active (by getting the config), and if this participant is even still speaker.
    /// If those things are true, state_machine's select_next gets called without a nomination.
    #[tracing::instrument(name = "automod_on_expired", skip(self, ctx))]
    async fn on_expired_event(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        let guard = ctx.volatile.room_locking().lock_room(self.room).await?;

        let result = self.on_expired_event_inner(&mut ctx).await;

        ctx.volatile.room_locking().unlock_room(guard).await?;
        result
    }

    async fn on_expired_event_inner(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        let storage = ctx.volatile.storage();
        let config = storage.config_get(self.room).await?;
        if let Some(config) = config {
            let speaker = storage.speaker_get(self.room).await?;

            if speaker == Some(self.id) {
                self.select_next(ctx, config, None).await?;
            }
        }
        Ok(())
    }

    /// Called whenever the timer for the current animation ends
    #[tracing::instrument(name = "automod_on_expired", skip(self, ctx))]
    async fn on_animation_end(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        selection: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        let guard = ctx.volatile.room_locking().lock_room(self.room).await?;

        let result = self.on_animation_end_inner(&mut ctx, selection).await;

        ctx.volatile.room_locking().unlock_room(guard).await?;
        result
    }

    async fn on_animation_end_inner(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        selection: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        let storage = ctx.volatile.storage();
        let config = storage.config_get(self.room).await?;
        if let Some(config) = config {
            let speaker = storage.speaker_get(self.room).await?;
            if speaker.is_none() {
                self.select_specific(ctx, config, selection, false).await?;
            }
        }
        Ok(())
    }

    async fn add_to_remaining(
        &mut self,
        storage: &mut dyn AutomodStorage,
        selection_strategy: SelectionStrategy,
    ) -> Result<(), SignalingModuleError> {
        if selection_strategy.uses_allow_list() {
            storage.allow_list_add(self.room, self.id).await?;
        } else {
            storage.playlist_push(self.room, self.id).await?;
        }

        Ok(())
    }

    /// Retrieves the remaining participants list.
    /// The storage mutex must be locked when calling this method.
    async fn get_remaining(
        &mut self,
        storage: &mut dyn AutomodStorage,
        selection_strategy: SelectionStrategy,
    ) -> Result<Vec<ParticipantId>, SignalingModuleError> {
        Ok(if selection_strategy.uses_allow_list() {
            storage
                .allow_list_get_all(self.room)
                .await?
                .into_iter()
                .collect()
        } else {
            storage.playlist_get_all(self.room).await?
        })
    }

    async fn validate_lists(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        parameter: &Parameter,
        allow_list: Option<&[ParticipantId]>,
        playlist: Option<&[ParticipantId]>,
    ) -> Result<bool, SignalingModuleError> {
        let mut allow_list_valid = true;
        let mut playlist_valid = true;

        if parameter.selection_strategy.uses_allow_list() {
            match allow_list {
                Some([]) => {
                    allow_list_valid = false;
                }
                Some(allow_list) => {
                    allow_list_valid = ctx
                        .volatile
                        .storage()
                        .check_participants_exist(self.room, allow_list)
                        .await?;
                }
                _ => allow_list_valid = false,
            }
        } else {
            match playlist {
                // TODO whenever hand raise is considered this must be updated to also check the allowlist
                Some([]) => {
                    playlist_valid = false;
                }
                Some(playlist) => {
                    playlist_valid = ctx
                        .volatile
                        .storage()
                        .check_participants_exist(self.room, playlist)
                        .await?;
                }
                _ => playlist_valid = false,
            }
        }

        if !(allow_list_valid && playlist_valid) {
            ctx.ws_send(AutomodEvent::Error(Error::InvalidSelection));
            Ok(false)
        } else {
            Ok(true)
        }
    }

    #[tracing::instrument(name = "automod_on_ws_message", skip(self, ctx, msg))]
    async fn on_ws_message(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        msg: AutomodCommand,
    ) -> Result<(), SignalingModuleError> {
        match ctx.role() {
            Role::User | Role::Guest => {
                if msg.requires_moderator_privileges() {
                    ctx.ws_send(AutomodEvent::Error(Error::InsufficientPermissions));

                    return Ok(());
                }
            }
            Role::Moderator => {}
        }
        let mut volatile = ctx.volatile.clone();
        let storage = volatile.storage();

        match msg {
            AutomodCommand::Start(Start {
                parameter,
                allow_list,
                playlist,
            }) => {
                if !self
                    .validate_lists(
                        &mut ctx,
                        &parameter,
                        allow_list.as_deref(),
                        playlist.as_deref(),
                    )
                    .await?
                {
                    return Ok(());
                }

                let guard = ctx.volatile.room_locking().lock_room(self.room).await?;

                let is_active = try_or_unlock!(
                    storage.config_exists( self.room).await;
                    ctx,
                    guard
                );

                if is_active {
                    ctx.volatile.room_locking().unlock_room(guard).await?;
                    ctx.ws_send(AutomodEvent::Error(Error::SessionAlreadyRunning));
                    return Ok(());
                }

                let config = StorageConfig::new(self.id, parameter);

                let remaining = match (
                    config.parameter.selection_strategy.uses_allow_list(),
                    allow_list,
                    playlist,
                ) {
                    (true, Some(allow_list), _) => {
                        try_or_unlock!(storage.allow_list_set(self.room, &allow_list).await; ctx, guard);
                        allow_list
                    }
                    (false, _, Some(playlist)) => {
                        try_or_unlock!(storage.playlist_set(self.room, &playlist).await; ctx, guard);
                        playlist
                    }
                    _ => {
                        // This should never happen.
                        //
                        // If it does check the `validate_lists` function.
                        ctx.volatile.room_locking().unlock_room(guard).await?;
                        // TODO: This should be a explicit failure point
                        // (Error::InvalidConfiguration).
                        return Ok(());
                    }
                };

                try_or_unlock!(
                    storage.config_set(self.room, config.clone()).await;
                    ctx,
                    guard
                );

                ctx.exchange_publish(
                    control::exchange::current_room_all_participants(self.room),
                    exchange::Message::Start(exchange::Start {
                        frontend_config: FrontendConfig {
                            parameter: config.parameter,
                            history: vec![],
                            remaining,
                            issued_by: config.issued_by,
                        },
                    }),
                );

                ctx.volatile.room_locking().unlock_room(guard).await?;
            }

            AutomodCommand::Edit(edit) => {
                let guard = ctx.volatile.room_locking().lock_room(self.room).await?;

                let config = try_or_unlock!(storage.config_get( self.room).await; ctx, guard);

                // only edit if automod is active
                if let Some(config) = config {
                    if !try_or_unlock!(self
                        .validate_lists(
                            &mut ctx,
                            &config.parameter,
                            edit.allow_list.as_deref(),
                            edit.playlist.as_deref(),
                        )
                        .await; ctx, guard)
                    {
                        ctx.volatile.room_locking().unlock_room(guard).await?;
                        return Ok(());
                    }

                    // set playlist if requested
                    if let Some(playlist) = &edit.playlist {
                        try_or_unlock!(
                            storage.playlist_set( self.room, playlist).await;
                            ctx,
                            guard
                        );
                    }

                    // set allow_list if requested
                    if let Some(allow_list) = &edit.allow_list {
                        try_or_unlock!(
                            storage.allow_list_set( self.room, allow_list).await;
                            ctx,
                            guard
                        );
                    }

                    // depending on the strategy find out if the remaining-list has been changed
                    let remaining = if config.parameter.selection_strategy.uses_allow_list() {
                        edit.allow_list
                    } else {
                        edit.playlist
                    };

                    // publish remaining update if remaining changed
                    if let Some(remaining) = remaining {
                        ctx.exchange_publish(
                            control::exchange::current_room_all_participants(self.room),
                            exchange::Message::RemainingUpdate(exchange::RemainingUpdate {
                                remaining,
                            }),
                        );
                    }
                }

                ctx.volatile.room_locking().unlock_room(guard).await?;
            }

            AutomodCommand::Stop => {
                let guard = ctx.volatile.room_locking().lock_room(self.room).await?;
                try_or_unlock!(
                    self.stop_session(
                        &mut ctx,
                        StoppedReason::StoppedByModerator { issued_by: self.id },
                    )
                    .await;
                    ctx,
                    guard
                );
                ctx.volatile.room_locking().unlock_room(guard).await?;
            }

            AutomodCommand::Select(select) => {
                let guard = ctx.volatile.room_locking().lock_room(self.room).await?;
                let config = try_or_unlock!(storage.config_get( self.room).await; ctx, guard);

                if let Some(config) = config {
                    match select {
                        Select::None => {
                            try_or_unlock!(
                                self.select_none(&mut ctx, config).await;
                                ctx,
                                guard
                            );
                        }
                        Select::Random => {
                            try_or_unlock!(
                                self.select_random(&mut ctx, config).await;
                                ctx,
                                guard
                            );
                        }
                        Select::Next => {
                            try_or_unlock!(
                                self.select_next(&mut ctx, config, None).await;
                                ctx,
                                guard
                            );
                        }
                        Select::Specific(specific) => {
                            try_or_unlock!(
                                self.select_specific(&mut ctx, config, specific.participant, specific.keep_in_remaining).await;
                                ctx,
                                guard
                            );
                        }
                    }
                }

                ctx.volatile.room_locking().unlock_room(guard).await?;
            }
            AutomodCommand::Yield(Yield { next }) => {
                let guard = ctx.volatile.room_locking().lock_room(self.room).await?;
                let config = try_or_unlock!(storage.config_get( self.room).await; ctx, guard);

                if let Some(config) = config {
                    let speaker = try_or_unlock!(storage.speaker_get( self.room).await; ctx, guard);

                    // Check if current speaker is self
                    if speaker != Some(self.id) {
                        ctx.ws_send(AutomodEvent::Error(Error::InsufficientPermissions));

                        ctx.volatile.room_locking().unlock_room(guard).await?;

                        return Ok(());
                    }

                    try_or_unlock!(self.select_next(&mut ctx, config, next).await; ctx, guard);
                }

                ctx.volatile.room_locking().unlock_room(guard).await?;
            }
        }

        Ok(())
    }

    /// Stops the current session, clearing up stored data.
    /// The storage mutex must be locked when calling this method.
    async fn stop_session(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        stopped_reason: StoppedReason,
    ) -> Result<(), SignalingModuleError> {
        let storage = ctx.volatile.storage();
        let config = storage.config_get(self.room).await?;

        if config.is_some() {
            storage.config_delete(self.room).await?;
            storage.allow_list_delete(self.room).await?;
            storage.playlist_delete(self.room).await?;

            ctx.exchange_publish(
                control::exchange::current_room_all_participants(self.room),
                exchange::Message::Stop(stopped_reason),
            );
        }

        Ok(())
    }

    /// Unselects the current speaker!!11elf
    async fn select_none(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        config: StorageConfig,
    ) -> Result<(), SignalingModuleError> {
        let result = state_machine::map_select_unchecked(
            state_machine::select_unchecked(ctx.volatile.storage(), self.room, &config, None).await,
        );

        self.handle_selection_result(ctx, config, result).await
    }

    /// Selects a specific participant to be the next speaker. Will check the participant if
    /// selection is valid.
    #[tracing::instrument(name = "automod_select_specific", skip(self, ctx, config))]
    async fn select_specific(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        config: StorageConfig,
        participant: ParticipantId,
        keep_in_remaining: bool,
    ) -> Result<(), SignalingModuleError> {
        let storage = ctx.volatile.storage();
        let is_valid = storage
            .participants_contains(self.room, participant)
            .await?;

        if !is_valid {
            return Ok(());
        }

        if !keep_in_remaining {
            if config.parameter.selection_strategy.uses_allow_list() {
                storage.allow_list_remove(self.room, participant).await?;
            } else {
                storage
                    .playlist_remove_first(self.room, participant)
                    .await?;
            }
        }

        let result = state_machine::map_select_unchecked(
            state_machine::select_unchecked(storage, self.room, &config, Some(participant)).await,
        );

        self.handle_selection_result(ctx, config, result).await
    }

    /// Selects a random participant to be the next speaker.
    #[tracing::instrument(name = "automod_select_random", skip(self, ctx, config))]
    async fn select_random(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        config: StorageConfig,
    ) -> Result<(), SignalingModuleError> {
        let result = state_machine::select_random(
            ctx.volatile.storage(),
            self.room,
            &config,
            &mut rand::rng(),
        )
        .await;

        self.handle_selection_result(ctx, config, result).await
    }

    #[tracing::instrument(name = "automod_select_next", skip(self, ctx, config))]
    async fn select_next(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        config: StorageConfig,
        nominated: Option<ParticipantId>,
    ) -> Result<(), SignalingModuleError> {
        let result = state_machine::select_next(
            ctx.volatile.storage(),
            self.room,
            &config,
            nominated,
            &mut rand::rng(),
        )
        .await;

        self.handle_selection_result(ctx, config, result).await
    }

    async fn handle_selection_result(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        config: StorageConfig,
        result: Result<Option<StateMachineOutput>, state_machine::Error>,
    ) -> Result<(), SignalingModuleError> {
        match result {
            Ok(Some(StateMachineOutput::SpeakerUpdate(update))) => {
                if update.speaker.is_some() {
                    ctx.exchange_publish(
                        control::exchange::current_room_all_participants(self.room),
                        exchange::Message::SpeakerUpdate(update),
                    );
                } else {
                    self.stop_session(ctx, StoppedReason::SessionFinished)
                        .await?;
                }

                Ok(())
            }
            Ok(Some(StateMachineOutput::StartAnimation(start_animation))) => {
                let animation_id = AnimationId(Uuid::new_v4());
                self.current_animation_id = Some(animation_id);

                let result = start_animation.result;
                ctx.add_event_stream(once(
                    sleep(Duration::from_secs(8))
                        .map(move |_| TimerEvent::AnimationEnd(animation_id, result)),
                ));

                // Reset current speaker, cannot use self.select_none() as that would end in a recursive function
                let update = state_machine::select_unchecked(
                    ctx.volatile.storage(),
                    self.room,
                    &config,
                    None,
                )
                .await
                .whatever_context::<_, SignalingModuleError>("Failed to reset speaker")?;

                if let Some(update) = update {
                    ctx.exchange_publish(
                        control::exchange::current_room_all_participants(self.room),
                        exchange::Message::SpeakerUpdate(update),
                    );
                }

                ctx.exchange_publish(
                    control::exchange::current_room_all_participants(self.room),
                    exchange::Message::StartAnimation(start_animation),
                );

                Ok(())
            }
            Ok(None) => {
                self.stop_session(ctx, StoppedReason::SessionFinished)
                    .await?;
                Ok(())
            }
            Err(state_machine::Error::InvalidSelection) => {
                ctx.ws_send(AutomodEvent::Error(Error::InvalidSelection));

                Ok(())
            }
            Err(state_machine::Error::Fatal { message, source }) => {
                Err(SignalingModuleError::CustomError { message, source })
            }
        }
    }

    #[tracing::instrument(name = "automod_on_exchange_msg", skip(self, ctx, msg))]
    async fn on_exchange_msg(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        msg: exchange::Message,
    ) -> Result<(), SignalingModuleError> {
        match msg {
            exchange::Message::Start(exchange::Start { frontend_config }) => {
                ctx.ws_send(AutomodEvent::Started(frontend_config.into_public()));
            }
            exchange::Message::Stop(stopped_reason) => {
                ctx.ws_send(AutomodEvent::Stopped(stopped_reason));
            }
            exchange::Message::SpeakerUpdate(exchange::SpeakerUpdate {
                speaker,
                history,
                remaining,
            }) => {
                // check if new speaker is self, then check for time limit
                if speaker
                    .map(|speaker| speaker == self.id)
                    .unwrap_or_default()
                {
                    let guard = ctx.volatile.room_locking().lock_room(self.room).await?;
                    let config = try_or_unlock!(ctx.volatile.storage().config_get( self.room).await; ctx, guard);

                    // There is a config and it contains a time limit.
                    // Add sleep "stream-future" which indicates the expiration of the speaker status
                    if let Some(time_limit) = config.and_then(|config| config.parameter.time_limit)
                    {
                        let expiry_id = ExpiryId(Uuid::new_v4());
                        self.current_expiry_id = Some(expiry_id);

                        ctx.add_event_stream(once(
                            sleep(time_limit).map(move |_| TimerEvent::Expiry(expiry_id)),
                        ));
                    }

                    ctx.volatile.room_locking().unlock_room(guard).await?;
                } else {
                    self.current_expiry_id = None;
                    self.current_animation_id = None;
                }

                ctx.ws_send(AutomodEvent::SpeakerUpdated(SpeakerUpdated {
                    speaker,
                    history,
                    remaining,
                }));
            }
            exchange::Message::RemainingUpdate(exchange::RemainingUpdate { remaining }) => {
                ctx.ws_send(AutomodEvent::RemainingUpdated(RemainingUpdated {
                    remaining,
                }));
            }
            exchange::Message::StartAnimation(start_animation) => {
                ctx.ws_send(AutomodEvent::StartAnimation(StartAnimation {
                    pool: start_animation.pool,
                    result: start_animation.result,
                }));
            }
        }

        Ok(())
    }
}
