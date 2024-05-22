// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod recording_storage;
mod redis;
mod volatile;

pub(crate) use recording_storage::RecordingStorage;
pub(super) use redis::{
    delete_all_streams, stream_exists, streams_contains_status, update_streams,
};
pub(crate) use redis::{get_stream, get_streams, set_stream, set_streams};

#[cfg(test)]
mod test_common {
    use opentalk_signaling_core::SignalingRoomId;

    use super::RecordingStorage;

    pub const ROOM: SignalingRoomId = SignalingRoomId::nil();

    pub(super) async fn streams(storage: &mut dyn RecordingStorage) {
        assert!(!storage.is_streaming_initialized(ROOM).await.unwrap());
    }
}
