// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_signaling_core::SignalingRoomId;

#[derive(Debug, Clone, Default)]
pub(crate) struct MemorySharedFolderState {
    initialized: BTreeSet<SignalingRoomId>,
}

impl MemorySharedFolderState {
    #[cfg(test)]
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn set_shared_folder_initialized(&mut self, room: SignalingRoomId) {
        self.initialized.insert(room);
    }

    pub(super) fn is_shared_folder_initialized(&self, room: SignalingRoomId) -> bool {
        self.initialized.contains(&room)
    }

    pub(super) fn delete_shared_folder_initialized(&mut self, room: SignalingRoomId) {
        self.initialized.remove(&room);
    }
}
