// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::VolatileStorage;

/// Context passed to the `destroy` function
pub struct DestroyContext<'ctx> {
    pub volatile: &'ctx mut VolatileStorage,
    pub cleanup_scope: CleanupScope,
}

impl DestroyContext<'_> {
    /// Returns true if the module belongs to the last participant inside a room
    pub fn destroy_room(&self) -> bool {
        self.cleanup_scope.destroy_room()
    }
}

/// The scope of the cleanup
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CleanupScope {
    /// Keep the rooms state, only the cleanup the runners context
    None,
    /// The current room state must be cleaned up. Global state must be kept if this is a breakout room.
    Local,
    /// All state that is related to the room must be cleaned up (including the main room if this is a breakout room)
    Global,
}

impl CleanupScope {
    /// Returns true when either the global or current room must be destroyed
    pub fn destroy_room(&self) -> bool {
        match self {
            CleanupScope::None => false,
            CleanupScope::Global | CleanupScope::Local => true,
        }
    }

    /// Returns true when the current room does not need a state clean up
    pub fn keep_room(&self) -> bool {
        !self.destroy_room()
    }
}
