// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use signaling_core::SignalingModuleInitData;
use signaling_core::{DestroyContext, Event, InitContext, ModuleContext, SignalingModule};

pub const CALL_IN_FEATURE: &str = "call_in";

pub struct Core {}

#[async_trait::async_trait(?Send)]
impl SignalingModule for Core {
    const NAMESPACE: &'static str = "core";

    type Params = ();
    type Incoming = ();
    type Outgoing = ();
    type ExchangeMessage = ();
    type ExtEvent = ();
    type FrontendData = ();
    type PeerFrontendData = ();

    async fn init(
        _ctx: InitContext<'_, Self>,
        _params: &Self::Params,
        _protocol: &'static str,
    ) -> Result<Option<Self>> {
        Ok(Some(Self {}))
    }

    fn get_provided_features() -> Vec<&'static str> {
        vec![CALL_IN_FEATURE]
    }

    async fn on_event(
        &mut self,
        mut _ctx: ModuleContext<'_, Self>,
        _event: Event<'_, Self>,
    ) -> Result<()> {
        Ok(())
    }

    async fn on_destroy(self, _ctx: DestroyContext<'_>) {}

    async fn build_params(_init: &SignalingModuleInitData) -> Result<Option<Self::Params>> {
        Ok(Some(()))
    }
}
