// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::fmt;
use types::core::{BreakoutRoomId, RoomId};

pub(crate) mod resumption;
pub(crate) mod ticket;

mod ws;
mod ws_modules;

pub use ws::module_tester::{ModuleTester, NoInitError, WsMessageOutgoing};
pub(crate) use ws::ws_service;
pub use ws::{
    Event, InitContext, ModuleContext, SignalingModule, SignalingModules, SignalingProtocols,
};
pub use ws_modules::{breakout, control, moderation, recording};

/// The complete room id
///
/// It consist of the room-id inside the database and an optional
/// breakout-room-id which is generated when the breakout rooms are created
#[derive(Debug, Copy, Clone)]
pub struct SignalingRoomId(RoomId, Option<BreakoutRoomId>);

impl SignalingRoomId {
    pub const fn new_test(room: RoomId) -> Self {
        Self(room, None)
    }

    pub const fn room_id(&self) -> RoomId {
        self.0
    }

    pub const fn breakout_room_id(&self) -> Option<BreakoutRoomId> {
        self.1
    }
}

impl fmt::Display for SignalingRoomId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(breakout) = self.1 {
            write!(f, "{}:{}", self.0, breakout)
        } else {
            self.0.fmt(f)
        }
    }
}
