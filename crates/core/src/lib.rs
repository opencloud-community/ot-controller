// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::{
    DestroyContext, Event, InitContext, ModuleContext, SignalingModule, SignalingModuleError,
    SignalingModuleInitData, VolatileStorageBackend,
};
use opentalk_types::signaling::core::{CALL_IN_FEATURE, NAMESPACE};

pub struct Core {}

pub struct VolatileWrapper {}

impl From<VolatileStorageBackend> for VolatileWrapper {
    fn from(_storage: VolatileStorageBackend) -> Self {
        Self {}
    }
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for Core {
    const NAMESPACE: &'static str = NAMESPACE;

    type Params = ();
    type Incoming = ();
    type Outgoing = ();
    type ExchangeMessage = ();
    type ExtEvent = ();
    type FrontendData = ();
    type PeerFrontendData = ();

    type Volatile = VolatileWrapper;

    async fn init(
        _ctx: InitContext<'_, Self>,
        _params: &Self::Params,
        _protocol: &'static str,
    ) -> Result<Option<Self>, SignalingModuleError> {
        Ok(Some(Self {}))
    }

    fn get_provided_features() -> Vec<&'static str> {
        vec![CALL_IN_FEATURE]
    }

    async fn on_event(
        &mut self,
        mut _ctx: ModuleContext<'_, Self>,
        _event: Event<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        Ok(())
    }

    async fn on_destroy(self, _ctx: DestroyContext<'_>) {}

    async fn build_params(
        _init: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
        Ok(Some(()))
    }
}
