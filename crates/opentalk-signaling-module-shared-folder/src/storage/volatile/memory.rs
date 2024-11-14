// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet};

use opentalk_signaling_core::SignalingRoomId;
use opentalk_types_common::shared_folders::SharedFolder;

#[derive(Debug, Clone, Default)]
pub(crate) struct MemorySharedFolderState {
    initialized: BTreeSet<SignalingRoomId>,
    shared_folders: BTreeMap<SignalingRoomId, SharedFolder>,
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

    pub(super) fn get_shared_folder(&self, room: SignalingRoomId) -> Option<SharedFolder> {
        self.shared_folders.get(&room).cloned()
    }

    pub(super) fn set_shared_folder(&mut self, room: SignalingRoomId, value: SharedFolder) {
        self.shared_folders.insert(room, value);
    }

    pub(super) fn delete_shared_folder(&mut self, room: SignalingRoomId) {
        self.shared_folders.remove(&room);
    }
}
