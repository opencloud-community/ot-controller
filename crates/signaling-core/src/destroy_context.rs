// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::RedisConnection;

/// Context passed to the `destroy` function
pub struct DestroyContext<'ctx> {
    pub redis_conn: &'ctx mut RedisConnection,
    pub destroy_room: bool,
}

impl DestroyContext<'_> {
    /// Access to a redis connection
    pub fn redis_conn(&mut self) -> &mut RedisConnection {
        self.redis_conn
    }

    /// Returns true if the module belongs to the last participant inside a room
    pub fn destroy_room(&self) -> bool {
        self.destroy_room
    }
}
