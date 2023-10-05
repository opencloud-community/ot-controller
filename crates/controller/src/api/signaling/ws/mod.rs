// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use signaling_core::{
    control, DestroyContext, ExchangeBinding, ExchangePublish, ModuleContext, SignalingModule,
};
use types::{
    core::Timestamp,
    signaling::{NamespacedCommand, NamespacedEvent},
};

mod actor;
mod http;
mod modules;
mod runner;

pub use http::ws_service;
pub use http::SignalingModules;
pub use http::SignalingProtocols;

pub(crate) trait ModuleContextExt {
    fn exchange_publish_control(
        &mut self,
        routing_key: String,
        message: control::exchange::Message,
    );
}

impl<M> ModuleContextExt for ModuleContext<'_, M>
where
    M: SignalingModule,
{
    /// Queue a outgoing control message
    ///
    /// Used in modules which control some behavior in the control module/runner
    fn exchange_publish_control(
        &mut self,
        routing_key: String,
        message: control::exchange::Message,
    ) {
        self.exchange_publish_any(
            routing_key,
            NamespacedEvent {
                namespace: control::NAMESPACE,
                timestamp: self.timestamp(),
                payload: message,
            },
        );
    }
}
