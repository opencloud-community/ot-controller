// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to the API endpoints under `/turn`.

mod ice_server;
mod stun_server;
mod turn_server;

pub use ice_server::IceServer;
pub use stun_server::StunServer;
pub use turn_server::TurnServer;
