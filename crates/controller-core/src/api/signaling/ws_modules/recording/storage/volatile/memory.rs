// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use opentalk_signaling_core::SignalingRoomId;
use opentalk_types::{core::StreamingTargetId, signaling::recording::StreamTargetSecret};

#[derive(Debug, Clone, Default)]
pub(super) struct MemoryRecordingState {
    streams: BTreeMap<SignalingRoomId, BTreeMap<StreamingTargetId, StreamTargetSecret>>,
}

impl MemoryRecordingState {
    #[cfg(test)]
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn is_streaming_initialized(&self, room: SignalingRoomId) -> bool {
        self.streams.contains_key(&room)
    }

    pub(super) fn set_streams(
        &mut self,
        room: SignalingRoomId,
        streams: &BTreeMap<StreamingTargetId, StreamTargetSecret>,
    ) {
        self.streams.insert(room, streams.clone());
    }

    pub(super) fn get_streams(
        &self,
        room: SignalingRoomId,
    ) -> BTreeMap<StreamingTargetId, StreamTargetSecret> {
        self.streams.get(&room).cloned().unwrap_or_default()
    }
}
