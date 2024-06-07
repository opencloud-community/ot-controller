// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::VolatileStorage;

/// Context passed to the `destroy` function
pub struct DestroyContext<'ctx> {
    pub volatile: &'ctx mut VolatileStorage,
    pub destroy_room: bool,
}

impl DestroyContext<'_> {
    /// Returns true if the module belongs to the last participant inside a room
    pub fn destroy_room(&self) -> bool {
        self.destroy_room
    }
}
