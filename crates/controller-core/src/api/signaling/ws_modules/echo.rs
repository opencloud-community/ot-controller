// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::{
    DestroyContext, Event, InitContext, ModuleContext, SignalingModule, SignalingModuleError,
    SignalingModuleInitData, VolatileStorageBackend,
};
use opentalk_types::signaling::echo::NAMESPACE;
use serde_json::Value;

#[derive(Clone)]
pub struct VolatileWrapper;
impl From<VolatileStorageBackend> for VolatileWrapper {
    fn from(_: VolatileStorageBackend) -> Self {
        Self
    }
}

/// A sample echo websocket module
pub struct Echo;

#[async_trait::async_trait(?Send)]
impl SignalingModule for Echo {
    const NAMESPACE: &'static str = NAMESPACE;
    type Params = ();
    type Incoming = Value;
    type Outgoing = Value;
    type ExchangeMessage = ();
    type ExtEvent = ();
    type FrontendData = ();
    type PeerFrontendData = ();

    type Volatile = VolatileWrapper;

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
