// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashMap;

use opentalk_signaling_core::SignalingRoomId;

#[derive(Debug, Clone, Default)]
pub(crate) struct MemoryProtocolState {
    group_id: HashMap<SignalingRoomId, String>,
}

impl MemoryProtocolState {
    pub(crate) fn group_set(&mut self, room_id: SignalingRoomId, group_id: &str) {
        self.group_id.insert(room_id, group_id.to_string());
    }
}
