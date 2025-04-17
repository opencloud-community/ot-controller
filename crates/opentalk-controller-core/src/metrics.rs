// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use actix_http::{body::BoxBody, StatusCode};
use actix_web::{dev::PeerAddr, get, web::Data, HttpResponse};
use kustos::metrics::KustosMetrics;
use opentalk_controller_service::metrics::EndpointMetrics;
use opentalk_controller_settings::SettingsProvider;
use opentalk_database::DatabaseMetrics;
use opentalk_signaling_core::{RedisMetrics, SignalingMetrics};
use opentelemetry::{global, otel_error};
use opentelemetry_sdk::metrics::{
    new_view, Aggregation, Instrument, MetricError, SdkMeterProvider, Stream,
};
use prometheus::{Encoder, Registry, TextEncoder};
use snafu::{Backtrace, Snafu};

use crate::Result;

#[derive(Debug, Snafu)]
#[snafu(context(false))]
pub struct MetricViewError {
    source: MetricError,
    backtrace: Backtrace,
}

pub struct CombinedMetrics {
    registry: Registry,
    pub(super) endpoint: Arc<EndpointMetrics>,
    pub(super) signaling: Arc<SignalingMetrics>,
    pub(super) database: Arc<DatabaseMetrics>,
    pub(super) kustos: Arc<KustosMetrics>,
    pub(super) redis: Arc<RedisMetrics>,
}

impl CombinedMetrics {
    pub fn try_init() -> Result<Self, MetricViewError> {
        let registry = prometheus::Registry::new();
        let exporter = opentelemetry_prometheus::exporter()
            .with_registry(registry.clone())
            .build()?;

        let provider = SdkMeterProvider::builder()
            .with_reader(exporter)
            .with_view(new_view(
                Instrument::new().name("web.request_duration_seconds"),
                Stream::new().aggregation(Aggregation::ExplicitBucketHistogram {
                    boundaries: vec![0.005, 0.01, 0.25, 0.5, 1.0, 2.0],
                    record_min_max: false,
                }),
            )?)
            .with_view(new_view(
                Instrument::new().name("web.response_sizes_bytes"),
                Stream::new().aggregation(Aggregation::ExplicitBucketHistogram {
                    boundaries: vec![100.0, 1_000.0, 10_000.0, 100_000.0],
                    record_min_max: false,
                }),
            )?)
            .with_view(new_view(
                Instrument::new().name("signaling.runner_startup_time_seconds"),
                Stream::new().aggregation(Aggregation::ExplicitBucketHistogram {
                    boundaries: vec![0.01, 0.25, 0.5, 1.0, 2.0, 5.0],
                    record_min_max: false,
                }),
            )?)
            .with_view(new_view(
                Instrument::new().name("signaling.runner_destroy_time_seconds"),
                Stream::new().aggregation(Aggregation::ExplicitBucketHistogram {
                    boundaries: vec![0.01, 0.25, 0.5, 1.0, 2.0, 5.0],
                    record_min_max: false,
                }),
            )?)
            .with_view(new_view(
                Instrument::new().name("sql.execution_time_seconds"),
                Stream::new().aggregation(Aggregation::ExplicitBucketHistogram {
                    boundaries: vec![0.01, 0.05, 0.1, 0.25, 0.5],
                    record_min_max: false,
                }),
            )?)
            .with_view(new_view(
                Instrument::new().name("sql.dbpool_connections"),
                Stream::new().aggregation(Aggregation::Default),
            )?)
            .with_view(new_view(
                Instrument::new().name("sql.dbpool_connections_idle"),
                Stream::new().aggregation(Aggregation::Default),
            )?)
            .with_view(new_view(
                Instrument::new().name("kustos.enforce_execution_time_seconds"),
                Stream::new().aggregation(Aggregation::ExplicitBucketHistogram {
                    boundaries: vec![0.01, 0.05, 0.1, 0.25, 0.5],
                    record_min_max: false,
                }),
            )?)
            .with_view(new_view(
                Instrument::new().name("kustos.load_policy_execution_time_seconds"),
                Stream::new().aggregation(Aggregation::ExplicitBucketHistogram {
                    boundaries: vec![0.01, 0.05, 0.1, 0.25, 0.5],
                    record_min_max: false,
                }),
            )?)
            .with_view(new_view(
                Instrument::new().name("redis.command_execution_time_seconds"),
                Stream::new().aggregation(Aggregation::ExplicitBucketHistogram {
                    boundaries: vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5],
                    record_min_max: false,
                }),
            )?)
            .build();

        global::set_meter_provider(provider);
        let meter = global::meter("ot-controller");

        let endpoint = Arc::new(EndpointMetrics {
            request_durations: meter
                .f64_histogram("web.request_duration_seconds")
                .with_description("HTTP response time measured in actix-web middleware")
                .with_unit("seconds")
                .build(),
            response_sizes: meter
                .u64_histogram("web.response_sizes_bytes")
                .with_description(
                    "HTTP response size for sized responses measured in actix-web middleware",
                )
                .with_unit("bytes")
                .build(),
            issued_email_tasks_count: meter
                .u64_counter("web.issued_email_tasks_count")
                .with_description("Number of issued email tasks")
                .build(),
        });

        let signaling = Arc::new(SignalingMetrics {
            runner_startup_time: meter
                .f64_histogram("signaling.runner_startup_time_seconds")
                .with_description("Time the runner takes to initialize")
                .with_unit("seconds")
                .build(),
            runner_destroy_time: meter
                .f64_histogram("signaling.runner_destroy_time_seconds")
                .with_description("Time the runner takes to stop")
                .with_unit("seconds")
                .build(),
            created_rooms_count: meter
                .u64_counter("signaling.created_rooms_count")
                .with_description("Number of created rooms")
                .build(),
            destroyed_rooms_count: meter
                .u64_counter("signaling.destroyed_rooms_count")
                .with_description("Number of destroyed rooms")
                .build(),
            participants_count: meter
                .i64_up_down_counter("signaling.participants_count")
                .with_description("Number of participants")
                .build(),
            participants_with_audio_count: meter
                .i64_up_down_counter("signaling.participants_with_audio_count")
                .with_description("Number of participants with audio unmuted")
                .build(),
            participants_with_video_count: meter
                .i64_up_down_counter("signaling.participants_with_video_count")
                .with_description("Number of participants with video unmuted")
                .build(),
        });

        let database = Arc::new(DatabaseMetrics {
            sql_execution_time: meter
                .f64_histogram("sql.execution_time_seconds")
                .with_description("SQL execution time for a single diesel query")
                .with_unit("seconds")
                .build(),
            sql_error: meter
                .u64_counter("sql.errors_total")
                .with_description("Counter for total SQL query errors")
                .build(),
            dbpool_connections: meter
                .u64_histogram("sql.dbpool_connections")
                .with_description("Number of currently non-idling db connections")
                .build(),
            dbpool_connections_idle: meter
                .u64_histogram("sql.dbpool_connections_idle")
                .with_description("Number of currently idling db connections")
                .build(),
        });

        let kustos = Arc::new(KustosMetrics {
            enforce_execution_time: meter
                .f64_histogram("kustos.enforce_execution_time_seconds")
                .with_description("Execution time of kustos enforce")
                .with_unit("seconds")
                .build(),
            load_policy_execution_time: meter
                .f64_histogram("kustos.load_policy_execution_time_seconds")
                .with_description("Execution time of kustos load_policy")
                .with_unit("seconds")
                .build(),
        });

        let redis = Arc::new(RedisMetrics {
            command_execution_time: meter
                .f64_histogram("redis.command_execution_time_seconds")
                .with_description("Execution time of redis commands in seconds")
                .with_unit("seconds")
                .build(),
        });

        Ok(Self {
            registry,
            endpoint,
            signaling,
            database,
            kustos,
            redis,
        })
    }
}

#[get("/metrics")]
pub async fn metrics(
    settings: Data<SettingsProvider>,
    PeerAddr(peer_addr): PeerAddr,
    metrics: Data<CombinedMetrics>,
) -> HttpResponse {
    let settings = settings.get_raw();

    let allowed = &settings
        .metrics
        .allowlist
        .iter()
        .any(|allowed_net| allowed_net.contains(&peer_addr.ip()));

    if !allowed {
        return HttpResponse::new(StatusCode::FORBIDDEN);
    }

    let encoder = TextEncoder::new();
    let metric_families = metrics.registry.gather();
    let mut buf = Vec::new();
    if let Err(err) = encoder.encode(&metric_families[..], &mut buf) {
        otel_error!(name: "export_failure", error = err.to_string());
        return HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let response = String::from_utf8(buf).unwrap_or_default();

    HttpResponse::with_body(StatusCode::OK, BoxBody::new(response))
}
