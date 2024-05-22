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
pub(crate) use redis::{get_stream, get_streams, set_stream};

#[cfg(test)]
mod test_common {
    use std::collections::BTreeMap;

    use opentalk_signaling_core::SignalingRoomId;
    use opentalk_types::{
        common::streaming::StreamingTargetKind,
        core::StreamingTargetId,
        signaling::recording::{StreamKindSecret, StreamTargetSecret},
    };

    use super::RecordingStorage;

    pub const ROOM: SignalingRoomId = SignalingRoomId::nil();

    pub(super) async fn streams(storage: &mut dyn RecordingStorage) {
        let stream1_id = StreamingTargetId::generate();
        let stream2_id = StreamingTargetId::generate();

        let stream1 = StreamTargetSecret {
            name: "Recording".to_string(),
            kind: StreamKindSecret::Recording,
            status: opentalk_types::signaling::recording::StreamStatus::Active,
        };
        let stream2 = StreamTargetSecret {
            name: "Livestream".to_string(),
            kind: StreamKindSecret::Livestream(StreamingTargetKind::Custom {
                streaming_endpoint: "rtmp://example.com/stream".parse().unwrap(),
                streaming_key: "abcdefgh".parse().unwrap(),
                public_url: "https://example.com/stream".parse().unwrap(),
            }),
            status: opentalk_types::signaling::recording::StreamStatus::Paused,
        };

        let streams = BTreeMap::from_iter([(stream1_id, stream1), (stream2_id, stream2)]);

        assert!(!storage.is_streaming_initialized(ROOM).await.unwrap());

        storage.set_streams(ROOM, &streams).await.unwrap();

        assert!(storage.is_streaming_initialized(ROOM).await.unwrap());
    }
}
