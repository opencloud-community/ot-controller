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

    pub(super) fn set_stream(
        &mut self,
        room: SignalingRoomId,
        target: StreamingTargetId,
        stream_target: StreamTargetSecret,
    ) {
        self.streams
            .entry(room)
            .or_default()
            .insert(target, stream_target);
    }

    pub(super) fn get_streams(
        &self,
        room: SignalingRoomId,
    ) -> BTreeMap<StreamingTargetId, StreamTargetSecret> {
        self.streams.get(&room).cloned().unwrap_or_default()
    }

    pub(super) fn get_stream(
        &self,
        room: SignalingRoomId,
        target: StreamingTargetId,
    ) -> Option<StreamTargetSecret> {
        self.streams
            .get(&room)
            .and_then(|targets| targets.get(&target))
            .cloned()
    }

    pub(super) fn stream_exists(&self, room: SignalingRoomId, target: StreamingTargetId) -> bool {
        self.streams
            .get(&room)
            .map(|targets| targets.contains_key(&target))
            .unwrap_or_default()
    }

    pub(super) fn delete_all_streams(&mut self, room: SignalingRoomId) {
        self.streams.remove(&room);
    }
}
