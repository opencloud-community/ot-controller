// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use actix_http::{body::BoxBody, StatusCode};
use actix_web::{dev::PeerAddr, get, web::Data, HttpResponse, HttpResponseBuilder};
use itertools::Itertools as _;
use kustos::metrics::KustosMetrics;
use opentalk_controller_service::metrics::EndpointMetrics;
use opentalk_controller_settings::SettingsProvider;
use opentalk_database::DatabaseMetrics;
use opentalk_signaling_core::{RedisMetrics, SignalingMetrics};
use opentelemetry::{global, otel_error};
use opentelemetry_sdk::metrics::{MetricError, SdkMeterProvider};
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

        let provider_builder = SdkMeterProvider::builder().with_reader(exporter);
        let provider_builder = EndpointMetrics::append_views(provider_builder)?;
        let provider_builder = SignalingMetrics::append_views(provider_builder)?;
        let provider_builder = DatabaseMetrics::append_views(provider_builder)?;
        let provider_builder = KustosMetrics::append_views(provider_builder)?;
        let provider_builder = RedisMetrics::append_views(provider_builder)?;

        global::set_meter_provider(provider_builder.build());
        let meter = global::meter("ot-controller");

        let endpoint = Arc::new(EndpointMetrics::new(&meter));
        let signaling = Arc::new(SignalingMetrics::new(&meter));
        let database = Arc::new(DatabaseMetrics::new(&meter));
        let kustos = Arc::new(KustosMetrics::new(&meter));
        let redis = Arc::new(RedisMetrics::new(&meter));

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
    let settings = settings.get();

    let allowlist = &settings.metrics.allowlist;
    let allowed = allowlist
        .iter()
        .any(|allowed_net| allowed_net.contains(&peer_addr.ip()));

    if !allowed {
        if allowlist.is_empty() {
            log::debug!("An attempt to access the metrics endpoint from IP address {peer_addr} was denied. Access to the metrics endpoint has not been configured.");
        } else {
            let allowed_nets = allowlist.iter().map(|net| format!("\"{net}\"")).join(", ");
            log::debug!("An attempt to access the metrics endpoint from IP address {peer_addr} was denied. Access allowed from: {allowed_nets}.");
        }
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

    HttpResponseBuilder::new(StatusCode::OK)
        .content_type("text/plain")
        .body(BoxBody::new(response))
}
