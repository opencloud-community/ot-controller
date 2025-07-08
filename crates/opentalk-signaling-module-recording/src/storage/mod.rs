// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod recording_storage;
mod redis;
mod volatile;

pub use recording_storage::RecordingStorage;

#[cfg(test)]
mod test_common {
    use std::collections::{BTreeMap, BTreeSet};

    use opentalk_signaling_core::SignalingRoomId;
    use opentalk_types_common::streaming::{StreamingTargetId, StreamingTargetKind};
    use opentalk_types_signaling_recording::{StreamKindSecret, StreamStatus, StreamTargetSecret};

    use super::RecordingStorage;

    pub const ROOM: SignalingRoomId = SignalingRoomId::nil();

    pub(super) async fn streams(storage: &mut dyn RecordingStorage) {
        let stream1_id = StreamingTargetId::generate();
        let stream2_id = StreamingTargetId::generate();
        let stream3_id = StreamingTargetId::generate();

        let stream1 = StreamTargetSecret {
            name: "Recording".to_string(),
            kind: StreamKindSecret::Recording,
            status: StreamStatus::Active,
        };
        let stream2 = StreamTargetSecret {
            name: "Livestream 1".to_string(),
            kind: StreamKindSecret::Livestream(StreamingTargetKind::Custom {
                streaming_endpoint: "rtmp://example.com/stream".parse().unwrap(),
                streaming_key: "abcdefgh".parse().unwrap(),
                public_url: "https://example.com/stream1".parse().unwrap(),
            }),
            status: StreamStatus::Paused,
        };
        let stream3 = StreamTargetSecret {
            name: "Livestream 2".to_string(),
            kind: StreamKindSecret::Livestream(StreamingTargetKind::Custom {
                streaming_endpoint: "rtmp://example.com/stream".parse().unwrap(),
                streaming_key: "ijklmnop".parse().unwrap(),
                public_url: "https://example.com/stream2".parse().unwrap(),
            }),
            status: StreamStatus::Inactive,
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

    pub(super) async fn streams_contain_status(storage: &mut dyn RecordingStorage) {
        let stream1_id = StreamingTargetId::generate();
        let stream2_id = StreamingTargetId::generate();
        const ROOM: SignalingRoomId = SignalingRoomId::nil();

        let stream1 = StreamTargetSecret {
            name: "Recording".to_string(),
            kind: StreamKindSecret::Recording,
            status: StreamStatus::Active,
        };
        let stream2 = StreamTargetSecret {
            name: "Livestream 1".to_string(),
            kind: StreamKindSecret::Livestream(StreamingTargetKind::Custom {
                streaming_endpoint: "rtmp://example.com/stream".parse().unwrap(),
                streaming_key: "abcdefgh".parse().unwrap(),
                public_url: "https://example.com/stream1".parse().unwrap(),
            }),
            status: StreamStatus::Paused,
        };

        let streams =
            BTreeMap::from_iter([(stream1_id, stream1.clone()), (stream2_id, stream2.clone())]);

        storage.set_streams(ROOM, &streams).await.unwrap();

        assert!(
            !storage
                .streams_contain_status(ROOM, BTreeSet::from_iter([]))
                .await
                .unwrap()
        );
        assert!(
            storage
                .streams_contain_status(ROOM, BTreeSet::from_iter([StreamStatus::Active]))
                .await
                .unwrap()
        );
        assert!(
            storage
                .streams_contain_status(ROOM, BTreeSet::from_iter([StreamStatus::Paused]))
                .await
                .unwrap()
        );
        assert!(
            !storage
                .streams_contain_status(ROOM, BTreeSet::from_iter([StreamStatus::Inactive]))
                .await
                .unwrap()
        );
        assert!(
            storage
                .streams_contain_status(
                    ROOM,
                    BTreeSet::from_iter([
                        StreamStatus::Inactive,
                        StreamStatus::Starting,
                        StreamStatus::Paused
                    ])
                )
                .await
                .unwrap()
        );
        assert!(
            !storage
                .streams_contain_status(
                    ROOM,
                    BTreeSet::from_iter([StreamStatus::Inactive, StreamStatus::Starting])
                )
                .await
                .unwrap()
        );
    }

    pub(super) async fn update_streams_status(storage: &mut dyn RecordingStorage) {
        let stream1_id = StreamingTargetId::generate();
        let stream2_id = StreamingTargetId::generate();
        const ROOM: SignalingRoomId = SignalingRoomId::nil();

        let stream1 = StreamTargetSecret {
            name: "Recording".to_string(),
            kind: StreamKindSecret::Recording,
            status: StreamStatus::Active,
        };
        let stream2 = StreamTargetSecret {
            name: "Livestream 1".to_string(),
            kind: StreamKindSecret::Livestream(StreamingTargetKind::Custom {
                streaming_endpoint: "rtmp://example.com/stream".parse().unwrap(),
                streaming_key: "abcdefgh".parse().unwrap(),
                public_url: "https://example.com/stream1".parse().unwrap(),
            }),
            status: StreamStatus::Paused,
        };

        let streams =
            BTreeMap::from_iter([(stream1_id, stream1.clone()), (stream2_id, stream2.clone())]);
        storage.set_streams(ROOM, &streams).await.unwrap();

        assert_eq!(
            storage.get_stream(ROOM, stream1_id).await.unwrap().status,
            StreamStatus::Active
        );
        assert_eq!(
            storage.get_stream(ROOM, stream2_id).await.unwrap().status,
            StreamStatus::Paused
        );

        storage
            .update_streams_status(
                ROOM,
                &BTreeSet::from_iter([stream1_id]),
                StreamStatus::Inactive,
            )
            .await
            .unwrap();

        assert_eq!(
            storage.get_stream(ROOM, stream1_id).await.unwrap().status,
            StreamStatus::Inactive
        );
        assert_eq!(
            storage.get_stream(ROOM, stream2_id).await.unwrap().status,
            StreamStatus::Paused
        );

        storage
            .update_streams_status(
                ROOM,
                &BTreeSet::from_iter([stream1_id, stream2_id]),
                StreamStatus::Active,
            )
            .await
            .unwrap();

        assert_eq!(
            storage.get_stream(ROOM, stream1_id).await.unwrap().status,
            StreamStatus::Active
        );
        assert_eq!(
            storage.get_stream(ROOM, stream2_id).await.unwrap().status,
            StreamStatus::Active
        );
    }
}
