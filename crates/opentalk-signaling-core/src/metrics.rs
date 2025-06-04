// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentelemetry::{
    metrics::{Counter, Histogram, Meter, UpDownCounter},
    Key, KeyValue,
};
use opentelemetry_sdk::metrics::{
    new_view, Aggregation, Instrument, MeterProviderBuilder, MetricError, Stream,
};

use crate::Participant;

const STARTUP_SUCCESSFUL: Key = Key::from_static_str("successful");
const DESTROY_SUCCESSFUL: Key = Key::from_static_str("successful");
const PARTICIPATION_KIND: Key = Key::from_static_str("participation_kind");
const MEDIA_SESSION_TYPE: Key = Key::from_static_str("media_session_type");
const RUNNER_STARTUP_TIME: &str = "signaling.runner_startup_time_seconds";
const RUNNER_DESTROY_TIME: &str = "signaling.runner_destroy_time_seconds";
const CREATED_ROOMS: &str = "signaling.created_rooms_count";
const DESTROYED_ROOMS: &str = "signaling.destroyed_rooms_count";
const CREATED_BREAKOUT_ROOMS: &str = "signaling.created_breakout_rooms_count";
const DESTROYED_BREAKOUT_ROOMS: &str = "signaling.destroyed_breakout_rooms_count";
const PARTICIPANT_COUNT: &str = "signaling.participants_count";
const PARTICIPANT_WITH_AUDIO_COUNT: &str = "signaling.participants_with_audio_count";
const PARTICIPANT_WITH_VIDEO_COUNT: &str = "signaling.participants_with_video_count";

pub struct SignalingMetrics {
    pub runner_startup_time: Histogram<f64>,
    pub runner_destroy_time: Histogram<f64>,
    pub created_rooms_count: Counter<u64>,
    pub destroyed_rooms_count: Counter<u64>,

    pub created_breakout_rooms_count: Counter<u64>,
    pub destroyed_breakout_rooms_count: Counter<u64>,

    pub participants_count: UpDownCounter<i64>,
    pub participants_with_audio_count: UpDownCounter<i64>,
    pub participants_with_video_count: UpDownCounter<i64>,
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

    pub fn increment_created_rooms_count(&self) {
        self.created_rooms_count.add(1, &[]);
    }

    pub fn increment_destroyed_rooms_count(&self) {
        self.destroyed_rooms_count.add(1, &[]);
    }

    pub fn increment_created_breakout_rooms_count(&self) {
        self.created_breakout_rooms_count.add(1, &[]);
    }

    pub fn increment_destroyed_breakout_rooms_count(&self) {
        self.destroyed_breakout_rooms_count.add(1, &[]);
    }

    pub fn increment_participants_count<U>(&self, participant: &Participant<U>) {
        self.participants_count.add(
            1,
            &[KeyValue::new(PARTICIPATION_KIND, participant.as_kind_str())],
        );
    }

    pub fn decrement_participants_count<U>(&self, participant: &Participant<U>) {
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
