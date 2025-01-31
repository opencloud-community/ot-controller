// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::{
    DestroyContext, Event, InitContext, ModuleContext, SignalingModule, SignalingModuleError,
    SignalingModuleInitData,
};
use opentalk_types_common::modules::{module_id, ModuleId};
use serde_json::Value;

/// The namespace string for the signaling module
pub const MODULE_ID: ModuleId = module_id!("echo");

/// A sample echo websocket module
pub struct Echo;

#[async_trait::async_trait(?Send)]
impl SignalingModule for Echo {
    const NAMESPACE: ModuleId = MODULE_ID;
    type Params = ();
    type Incoming = Value;
    type Outgoing = Value;
    type ExchangeMessage = ();
    type ExtEvent = ();
    type FrontendData = ();
    type PeerFrontendData = ();

    async fn init(
        _: InitContext<'_, Self>,
        _: &Self::Params,
        _: &'static str,
    ) -> Result<Option<Self>, SignalingModuleError> {
        Ok(Some(Self))
    }

    async fn on_event(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        event: Event<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        match event {
            Event::WsMessage(incoming) => {
                ctx.ws_send(incoming);
            }
            Event::Exchange(_) => {}
            Event::Ext(_) => unreachable!("no registered external events"),
            // Ignore
            Event::RaiseHand => {}
            Event::LowerHand => {}
            Event::Joined { .. } => {}
            Event::Leaving => {}
            Event::ParticipantJoined(..) => {}
            Event::ParticipantLeft(_) => {}
            Event::ParticipantUpdated(..) => {}
            Event::RoleUpdated(_) => {}
        }

        Ok(())
    }

    async fn on_destroy(self, _: DestroyContext<'_>) {}

    async fn build_params(
        _init: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
        Ok(Some(()))
    }
}
