// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_signaling_core::{
    DestroyContext, Event, InitContext, ModuleContext, SignalingModule, SignalingModuleError,
    SignalingModuleInitData,
};
use opentalk_types_common::{
    features::{feature_id, FeatureId},
    modules::{module_id, ModuleId},
};

/// The namespace string for the signaling module
pub const MODULE_ID: ModuleId = module_id!("integration");

/// The identifier string for the outlook feature
pub const OUTLOOK_FEATURE_ID: FeatureId = feature_id!("outlook");

pub struct Integration;

#[async_trait::async_trait(?Send)]
impl SignalingModule for Integration {
    const NAMESPACE: ModuleId = MODULE_ID;

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
    ) -> Result<Option<Self>, SignalingModuleError> {
        Ok(Some(Self {}))
    }

    fn get_provided_features() -> BTreeSet<FeatureId> {
        BTreeSet::from_iter([OUTLOOK_FEATURE_ID])
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
