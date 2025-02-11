// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use async_trait::async_trait;
use opentalk_signaling_core::{
    control::{exchange, ControlStorageProvider},
    DestroyContext, Event, InitContext, ModuleContext, SignalingModule, SignalingModuleError,
    SignalingModuleInitData, SignalingRoomId,
};
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_e2ee::{
    E2eeCommand, E2eeEvent, Error, Invite, MlsMessages, WelcomeMessage,
};
use serde::{Deserialize, Serialize};

pub struct E2ee {
    room_id: SignalingRoomId,
    participant_id: ParticipantId,
}

#[async_trait(? Send)]
impl SignalingModule for E2ee {
    const NAMESPACE: ModuleId = opentalk_types_signaling_e2ee::MODULE_ID;

    type Params = ();
    type Incoming = E2eeCommand;
    type Outgoing = E2eeEvent;
    type ExchangeMessage = ExchangeMessage;
    type ExtEvent = ();
    type FrontendData = ();
    type PeerFrontendData = ();

    async fn init(
        ctx: InitContext<'_, Self>,
        _params: &Self::Params,
        _protocol: &'static str,
    ) -> Result<Option<Self>, SignalingModuleError> {
        Ok(Some(Self {
            room_id: ctx.room_id(),
            participant_id: ctx.participant_id(),
        }))
    }

    async fn on_event(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        event: Event<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        match event {
            Event::ParticipantLeft(participant_id) => {
                ctx.ws_send(E2eeEvent::ParticipantLeft { participant_id });
            }
            Event::WsMessage(command) => self.handle_command(&mut ctx, command).await?,
            Event::Exchange(message) => self.handle_exchange_message(&mut ctx, message),
            _ => {}
        }
        Ok(())
    }

    async fn on_destroy(self, mut _ctx: DestroyContext<'_>) {}

    async fn build_params(
        _init: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
        Ok(Some(()))
    }
}

impl E2ee {
    async fn handle_command(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        command: E2eeCommand,
    ) -> Result<(), SignalingModuleError> {
        match command {
            E2eeCommand::Invite(invite) => {
                if !self.invite_valid(ctx, &invite).await? {
                    return Ok(());
                }

                for participant in self.get_all_participants(ctx).await? {
                    if self.participant_id == participant || invite.invitee == participant {
                        continue;
                    }
                    ctx.exchange_publish(
                        exchange::current_room_by_participant_id(self.room_id, participant),
                        ExchangeMessage::MlsMessages(invite.mls_messages.clone()),
                    );
                }
                ctx.exchange_publish(
                    exchange::current_room_by_participant_id(self.room_id, invite.invitee),
                    ExchangeMessage::Welcome(invite.welcome_message),
                );
            }
            E2eeCommand::Message(mls_messages) => {
                if !mls_messages.is_valid() {
                    return Ok(());
                }

                self.notify_other_participants(ctx, ExchangeMessage::MlsMessages(mls_messages))
                    .await?;
            }
        }
        Ok(())
    }

    fn handle_exchange_message(&self, ctx: &mut ModuleContext<'_, Self>, message: ExchangeMessage) {
        match message {
            ExchangeMessage::Welcome(welcome) => ctx.ws_send(E2eeEvent::Welcome(welcome)),
            ExchangeMessage::MlsMessages(messages) => ctx.ws_send(E2eeEvent::MlsMessages(messages)),
        }
    }

    async fn notify_other_participants(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        message: ExchangeMessage,
    ) -> Result<(), SignalingModuleError> {
        for participant_id in self.get_all_participants(ctx).await? {
            if participant_id == self.participant_id {
                continue;
            }

            ctx.exchange_publish(
                exchange::current_room_by_participant_id(self.room_id, participant_id),
                message.clone(),
            );
        }
        Ok(())
    }

    async fn invite_valid(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        invite: &Invite,
    ) -> Result<bool, SignalingModuleError> {
        if !invite.welcome_message.is_valid() || !invite.mls_messages.is_valid() {
            ctx.ws_send(E2eeEvent::Error(Error::InvalidInvite));
            return Ok(false);
        }
        self.participant_id_valid(ctx, &invite.invitee).await
    }

    async fn participant_id_valid(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        id: &ParticipantId,
    ) -> Result<bool, SignalingModuleError> {
        let participants = self.get_all_participants(ctx).await?;
        if !participants.contains(id) {
            ctx.ws_send(E2eeEvent::Error(Error::InvalidParticipantTarget));
            return Ok(false);
        }
        Ok(true)
    }

    async fn get_all_participants(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<BTreeSet<ParticipantId>, SignalingModuleError> {
        ctx.volatile
            .control_storage()
            .get_all_participants(self.room_id)
            .await
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ExchangeMessage {
    Welcome(WelcomeMessage),
    MlsMessages(MlsMessages),
}
