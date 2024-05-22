// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod recording_storage;
mod redis;
mod volatile;

pub(crate) use recording_storage::RecordingStorage;
pub(super) use redis::{delete_all_streams, streams_contains_status, update_streams};

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
        let stream3_id = StreamingTargetId::generate();

        let stream1 = StreamTargetSecret {
            name: "Recording".to_string(),
            kind: StreamKindSecret::Recording,
            status: opentalk_types::signaling::recording::StreamStatus::Active,
        };
        let stream2 = StreamTargetSecret {
            name: "Livestream 1".to_string(),
            kind: StreamKindSecret::Livestream(StreamingTargetKind::Custom {
                streaming_endpoint: "rtmp://example.com/stream".parse().unwrap(),
                streaming_key: "abcdefgh".parse().unwrap(),
                public_url: "https://example.com/stream1".parse().unwrap(),
            }),
            status: opentalk_types::signaling::recording::StreamStatus::Paused,
        };
        let stream3 = StreamTargetSecret {
            name: "Livestream 2".to_string(),
            kind: StreamKindSecret::Livestream(StreamingTargetKind::Custom {
                streaming_endpoint: "rtmp://example.com/stream".parse().unwrap(),
                streaming_key: "ijklmnop".parse().unwrap(),
                public_url: "https://example.com/stream2".parse().unwrap(),
            }),
            status: opentalk_types::signaling::recording::StreamStatus::Inactive,
        };

        let streams =
            BTreeMap::from_iter([(stream1_id, stream1.clone()), (stream2_id, stream2.clone())]);

        assert!(!storage.is_streaming_initialized(ROOM).await.unwrap());
        assert_eq!(storage.get_streams(ROOM).await.unwrap(), BTreeMap::new());
        assert!(storage.get_stream(ROOM, stream1_id).await.is_err());
        assert!(storage.get_stream(ROOM, stream2_id).await.is_err());
        assert!(storage.get_stream(ROOM, stream3_id).await.is_err());
        assert!(!storage.stream_exists(ROOM, stream1_id).await.unwrap());
        assert!(!storage.stream_exists(ROOM, stream2_id).await.unwrap());
        assert!(!storage.stream_exists(ROOM, stream3_id).await.unwrap());

        storage.set_streams(ROOM, &streams).await.unwrap();
        assert_eq!(storage.get_stream(ROOM, stream1_id).await.unwrap(), stream1);
        assert_eq!(storage.get_stream(ROOM, stream2_id).await.unwrap(), stream2);
        assert!(storage.get_stream(ROOM, stream3_id).await.is_err());
        assert!(storage.stream_exists(ROOM, stream1_id).await.unwrap());
        assert!(storage.stream_exists(ROOM, stream2_id).await.unwrap());
        assert!(!storage.stream_exists(ROOM, stream3_id).await.unwrap());

        assert!(storage.is_streaming_initialized(ROOM).await.unwrap());

        assert_eq!(storage.get_streams(ROOM).await.unwrap(), streams);

        storage
            .set_stream(ROOM, stream3_id, stream3.clone())
            .await
            .unwrap();
        assert_eq!(
            storage.get_streams(ROOM).await.unwrap(),
            BTreeMap::from_iter([
                (stream1_id, stream1),
                (stream2_id, stream2),
                (stream3_id, stream3)
            ])
        );
    }
}
