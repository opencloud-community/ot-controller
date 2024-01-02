// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::header::USER_AGENT;
use actix_web::{Error, HttpMessage};
use anyhow::Result;
use controller_settings::Logging;
use opentelemetry::global;
use opentelemetry::sdk::{trace, Resource};
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use tracing::Span;
use tracing_actix_web::{RequestId, RootSpanBuilder};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};
use uuid::Uuid;

// If these default values are adjusted, that change should be synchronized
// into `extra/example.toml` for transparency towards administrators
// and documentation purposes.
const DEFAULT_LOGGING_DIRECTIVES: &str = "error,\
    opentalk=info,\
    pinky_swear=off,\
    rustls=warn,\
    mio=error,\
    lapin=warn";

pub fn init(settings: &Logging) -> Result<()> {
    // Layer which acts as filter of traces and spans.
    // The filter is created from enviroment (RUST_LOG) and config file
    let filter = create_filter(settings)?;

    // FMT layer prints the trace events into stdout
    let fmt = tracing_subscriber::fmt::Layer::default();

    // Create registry which contains all layers
    let registry = Registry::default().with(filter).with(fmt);

    // If opentelemetry is enabled install that layer
    if let Some(endpoint) = &settings.otlp_tracing_endpoint {
        let otlp_exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(endpoint);
        let service_name = settings
            .service_name
            .clone()
            .unwrap_or_else(|| "controller".into());
        let service_namespace = settings
            .service_namespace
            .clone()
            .unwrap_or_else(|| "opentalk".into());
        let service_instance_id = settings
            .service_instance_id
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(otlp_exporter)
            .with_trace_config(trace::config().with_resource(Resource::new(vec![
                    KeyValue::new("service.name", service_name),
                    KeyValue::new("service.namespace", service_namespace),
                    KeyValue::new("service.instance.id", service_instance_id),
                    KeyValue::new(
                        "service.version",
                        option_env!("VERGEN_GIT_SEMVER")
                            .or(option_env!("VERGEN_BUILD_SEMVER"))
                            .unwrap_or("unknown"),
                    ),
                ])))
            .install_batch(opentelemetry::runtime::TokioCurrentThread)?;
        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

        // Initialize the global logging with telemetry
        registry.with(telemetry).init();
    } else {
        // Install the global logger
        registry.init();
    }

    Ok(())
}

/// Create the logging filter
///
/// The filter is a combination of the values from the RUST_LOG environment variable, the config file and
/// the controllers defaults.
///
/// The priority of the different config options is RUST_LOG > config file > controller defaults.
fn create_filter(settings: &Logging) -> Result<EnvFilter> {
    // Read the config from the RUST_LOG environment variable
    let env_directives = std::env::var(EnvFilter::DEFAULT_ENV)
        .ok()
        .filter(|v| !v.is_empty());

    let config_directives = settings
        .default_directives
        .as_ref()
        .filter(|v| !v.is_empty());

    let mut directives = DEFAULT_LOGGING_DIRECTIVES.to_owned();

    if let Some(config_directives) = config_directives {
        directives = [directives, config_directives.join(",")].join(",")
    }

    if let Some(env_directives) = env_directives {
        directives = [directives, env_directives].join(",")
    }

    let filter = EnvFilter::new(directives);

    Ok(filter)
}

/// Flush remaining spans and traces
pub async fn destroy() {
    let handle = tokio::runtime::Handle::current();

    if handle
        .spawn_blocking(global::shutdown_tracer_provider)
        .await
        .is_err()
    {
        eprintln!(
            "Failed to shutdown opentelemetry tracer provider, some information might be missing"
        );
    }
}

pub struct ReducedSpanBuilder;

impl RootSpanBuilder for ReducedSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        create_span(request)
    }

    fn on_request_end<B>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        match &outcome {
            Ok(response) => {
                if let Some(error) = response.response().error() {
                    handle_error(span, error)
                } else {
                    span.record("http.status_code", response.response().status().as_u16());
                    span.record("otel.status_code", "OK");
                }
            }
            Err(error) => handle_error(span, error),
        };
    }
}

fn handle_error(span: Span, error: &Error) {
    let response_error = error.as_response_error();
    span.record(
        "exception.message",
        &tracing::field::display(response_error),
    );
    span.record("exception.details", &tracing::field::debug(response_error));
    let status_code = response_error.status_code();
    span.record("http.status_code", status_code.as_u16());

    if status_code.is_client_error() {
        span.record("otel.status_code", "OK");
    } else {
        span.record("otel.status_code", "ERROR");
    }
}

fn create_span(request: &ServiceRequest) -> Span {
    let user_agent = request
        .headers()
        .get(USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    let http_route: std::borrow::Cow<'static, str> = request
        .match_pattern()
        .map(Into::into)
        .unwrap_or_else(|| "default".into());

    let connection_info = request.connection_info();
    let request_id = request.extensions().get::<RequestId>().cloned().unwrap();
    let span = tracing::info_span!(
        "HTTP request",
        http.method = %request.method().as_str(),
        http.route = %http_route,
        http.flavor = ?request.version(),
        http.scheme = %connection_info.scheme(),
        http.host = %connection_info.host(),
        http.user_agent = %user_agent,
        http.target = %request.uri(),
        http.status_code = tracing::field::Empty,
        otel.kind = "server",
        otel.status_code = tracing::field::Empty,
        request_id = %request_id,
        trace_id = tracing::field::Empty,
        exception.message = tracing::field::Empty,
        // Not proper OpenTelemetry, but their terminology is fairly exception-centric
        exception.details = tracing::field::Empty,
    );

    span
}
