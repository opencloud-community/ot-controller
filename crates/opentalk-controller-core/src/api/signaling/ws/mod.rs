// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::{
    CleanupScope, DestroyContext, ExchangeBinding, ExchangePublish, ModuleContext, SignalingModule,
};

mod actor;
mod http;
mod modules;
mod runner;

pub use http::SignalingModules;
pub(crate) use http::{__path_ws_service, ws_service, SignalingProtocols};
use opentalk_types_signaling::NamespacedEvent;

pub enum RunnerMessage {
    Message(actix_web_actors::ws::Message),
    Timeout,
}
