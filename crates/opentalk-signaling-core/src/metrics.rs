// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::HashMap, time::Instant};

use opentalk_types_common::rooms::RoomId;
use opentalk_types_signaling::ParticipantId;
use opentelemetry::{
    metrics::{Counter, Histogram, Meter, UpDownCounter},
    Key, KeyValue,
};
use opentelemetry_sdk::metrics::{
    new_view, Aggregation, Instrument, MeterProviderBuilder, MetricError, Stream,
};
use parking_lot::Mutex;

use crate::Participant;

const STARTUP_SUCCESSFUL: Key = Key::from_static_str("successful");
const DESTROY_SUCCESSFUL: Key = Key::from_static_str("successful");
const PARTICIPATION_KIND: Key = Key::from_static_str("participation_kind");
const MEDIA_SESSION_TYPE: Key = Key::from_static_str("media_session_type");
const RUNNER_STARTUP_TIME: &str = "signaling.runner_startup_time_seconds";
const RUNNER_DESTROY_TIME: &str = "signaling.runner_destroy_time_seconds";
const ROOM_LIFE_TIME: &str = "signaling.room_life_time";
const CREATED_ROOMS: &str = "signaling.created_rooms_count";
const DESTROYED_ROOMS: &str = "signaling.destroyed_rooms_count";
const CREATED_BREAKOUT_ROOMS: &str = "signaling.created_breakout_rooms_count";
const DESTROYED_BREAKOUT_ROOMS: &str = "signaling.destroyed_breakout_rooms_count";
const PARTICIPANT_COUNT: &str = "signaling.participants_count";
const PARTICIPANT_WITH_AUDIO_COUNT: &str = "signaling.participants_with_audio_count";
const PARTICIPANT_WITH_VIDEO_COUNT: &str = "signaling.participants_with_video_count";
const PARTICIPANT_MEETING_TIME: &str = "signaling.participant_meeting_time";

pub struct SignalingMetrics {
    pub runner_startup_time: Histogram<f64>,
    pub runner_destroy_time: Histogram<f64>,
    pub room_life_time: Histogram<u64>,
    pub created_rooms_count: Counter<u64>,
    pub destroyed_rooms_count: Counter<u64>,

    pub created_breakout_rooms_count: Counter<u64>,
    pub destroyed_breakout_rooms_count: Counter<u64>,

    pub participants_count: UpDownCounter<i64>,
    pub participants_with_audio_count: UpDownCounter<i64>,
    pub participants_with_video_count: UpDownCounter<i64>,
    pub participant_meeting_time: Histogram<u64>,

    rooms: Mutex<HashMap<RoomId, Instant>>,
    participants: Mutex<HashMap<ParticipantId, Instant>>,
}

impl SignalingMetrics {
    pub fn append_views(
        provider_builder: MeterProviderBuilder,
    ) -> Result<MeterProviderBuilder, MetricError> {
        Ok(provider_builder
            .with_view(new_view(
                Instrument::new().name(RUNNER_STARTUP_TIME),
                Stream::new().aggregation(Aggregation::ExplicitBucketHistogram {
                    boundaries: vec![0.01, 0.25, 0.5, 1.0, 2.0, 5.0],
                    record_min_max: false,
                }),
            )?)
            .with_view(new_view(
                Instrument::new().name(RUNNER_DESTROY_TIME),
                Stream::new().aggregation(Aggregation::ExplicitBucketHistogram {
                    boundaries: vec![0.01, 0.25, 0.5, 1.0, 2.0, 5.0],
                    record_min_max: false,
                }),
            )?)
            .with_view(new_view(
                Instrument::new().name(ROOM_LIFE_TIME),
                Stream::new().aggregation(Aggregation::ExplicitBucketHistogram {
                    boundaries: vec![
                        2.0 * 60.0,        // 2 minutes
                        5.0 * 60.0,        // 5 minutes
                        30.0 * 60.0,       // 30 minutes
                        60.0 * 60.0,       // 1 hour
                        3.0 * 60.0 * 60.0, // 3 hours
                        8.0 * 60.0 * 60.0, // 8 hours
                    ],
                    record_min_max: false,
                }),
            )?)
            .with_view(new_view(
                Instrument::new().name(PARTICIPANT_MEETING_TIME),
                Stream::new().aggregation(Aggregation::ExplicitBucketHistogram {
                    boundaries: vec![
                        2.0 * 60.0,        // 2 minutes
                        5.0 * 60.0,        // 5 minutes
                        30.0 * 60.0,       // 30 minutes
                        60.0 * 60.0,       // 1 hour
                        3.0 * 60.0 * 60.0, // 3 hours
                        8.0 * 60.0 * 60.0, // 8 hours
                    ],
                    record_min_max: false,
                }),
            )?))
    }

    pub fn new(meter: &Meter) -> Self {
        Self {
            runner_startup_time: meter
                .f64_histogram(RUNNER_STARTUP_TIME)
                .with_description("Time the runner takes to initialize")
                .with_unit("seconds")
                .build(),
            runner_destroy_time: meter
                .f64_histogram(RUNNER_DESTROY_TIME)
                .with_description("Time the runner takes to stop")
                .with_unit("seconds")
                .build(),
            room_life_time: meter
                .u64_histogram(ROOM_LIFE_TIME)
                .with_description("Time rooms were active")
                .with_unit("seconds")
                .build(),
            created_rooms_count: meter
                .u64_counter(CREATED_ROOMS)
                .with_description("Number of created rooms")
                .build(),
            destroyed_rooms_count: meter
                .u64_counter(DESTROYED_ROOMS)
                .with_description("Number of destroyed rooms")
                .build(),
            created_breakout_rooms_count: meter
                .u64_counter(CREATED_BREAKOUT_ROOMS)
                .with_description("Number of created breakout rooms")
                .build(),
            destroyed_breakout_rooms_count: meter
                .u64_counter(DESTROYED_BREAKOUT_ROOMS)
                .with_description("Number of destroyed breakout rooms")
                .build(),
            participants_count: meter
                .i64_up_down_counter(PARTICIPANT_COUNT)
                .with_description("Number of participants")
                .build(),
            participants_with_audio_count: meter
                .i64_up_down_counter(PARTICIPANT_WITH_AUDIO_COUNT)
                .with_description("Number of participants with audio unmuted")
                .build(),
            participants_with_video_count: meter
                .i64_up_down_counter(PARTICIPANT_WITH_VIDEO_COUNT)
                .with_description("Number of participants with video unmuted")
                .build(),
            participant_meeting_time: meter
                .u64_histogram(PARTICIPANT_MEETING_TIME)
                .with_description("Time a participant spent in a meeting room")
                .build(),
            rooms: Mutex::new(HashMap::new()),
            participants: Mutex::new(HashMap::new()),
        }
    }

    pub fn record_startup_time(&self, secs: f64, success: bool) {
        self.runner_startup_time
            .record(secs, &[KeyValue::new(STARTUP_SUCCESSFUL, success)]);
    }

    pub fn record_destroy_time(&self, secs: f64, success: bool) {
        self.runner_destroy_time
            .record(secs, &[KeyValue::new(DESTROY_SUCCESSFUL, success)]);
    }

    pub fn record_room_creation_metrics(&self, room_id: RoomId) {
        let mut rooms = self.rooms.lock();
        if rooms.insert(room_id, Instant::now()).is_some() {
            log::warn!("room creation metrics invoked twice for room {room_id}. Skipping.");
            return;
        }
        self.created_rooms_count.add(1, &[]);
    }

    pub fn record_room_destroyed_metrics(&self, room_id: RoomId) {
        let mut rooms = self.rooms.lock();
        let Some(created_at) = rooms.remove(&room_id) else {
            log::warn!("room destroy metrics invoked for room {room_id}, that does not exist.");
            return;
        };
        let life_time = Instant::now().duration_since(created_at);
        self.room_life_time.record(life_time.as_secs(), &[]);
        self.destroyed_rooms_count.add(1, &[]);
    }

    pub fn increment_created_breakout_rooms_count(&self) {
        self.created_breakout_rooms_count.add(1, &[]);
    }

    pub fn increment_destroyed_breakout_rooms_count(&self) {
        self.destroyed_breakout_rooms_count.add(1, &[]);
    }

    pub fn record_participant_joined<U>(
        &self,
        participant: &Participant<U>,
        participant_id: ParticipantId,
    ) {
        if matches!(participant, Participant::Recorder) {
            return;
        }

        let mut participants = self.participants.lock();
        if participants
            .insert(participant_id, Instant::now())
            .is_some()
        {
            log::warn!("participant joined metrics invoked twice for participant {participant_id}");
            return;
        }
        self.participants_count.add(
            1,
            &[KeyValue::new(PARTICIPATION_KIND, participant.as_kind_str())],
        );
    }

    pub fn record_participant_left<U>(
        &self,
        participant: &Participant<U>,
        participant_id: ParticipantId,
    ) {
        if matches!(participant, Participant::Recorder) {
            return;
        }

        let mut participants = self.participants.lock();
        let Some(joined_at) = participants.remove(&participant_id) else {
            log::warn!("participant left metrics invoked for participant {participant_id} that does not exist");
            return;
        };
        let meeting_time = Instant::now().duration_since(joined_at);
        self.participant_meeting_time
            .record(meeting_time.as_secs(), &[]);
        self.participants_count.add(
            -1,
            &[KeyValue::new(PARTICIPATION_KIND, participant.as_kind_str())],
        );
    }

    pub fn increment_participants_with_audio_count(&self, session_type: &str) {
        self.participants_with_audio_count.add(
            1,
            &[KeyValue::new(MEDIA_SESSION_TYPE, session_type.to_owned())],
        );
    }

    pub fn decrement_participants_with_audio_count(&self, session_type: &str) {
        self.participants_with_audio_count.add(
            -1,
            &[KeyValue::new(MEDIA_SESSION_TYPE, session_type.to_owned())],
        );
    }

    pub fn increment_participants_with_video_count(&self, session_type: &str) {
        self.participants_with_video_count.add(
            1,
            &[KeyValue::new(MEDIA_SESSION_TYPE, session_type.to_owned())],
        );
    }

    pub fn decrement_participants_with_video_count(&self, session_type: &str) {
        self.participants_with_video_count.add(
            -1,
            &[KeyValue::new(MEDIA_SESSION_TYPE, session_type.to_owned())],
        );
    }
}
