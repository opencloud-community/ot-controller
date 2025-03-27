// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod ws;

pub use opentalk_controller_service::signaling::ws_modules::{
    breakout, echo, moderation, recording, recording_service,
};
pub use ws::SignalingModules;
pub(crate) use ws::{__path_ws_service, ws_service, SignalingProtocols};
