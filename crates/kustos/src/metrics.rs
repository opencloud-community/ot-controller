// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentelemetry::metrics::{Histogram, Meter};
use opentelemetry_sdk::metrics::{
    Aggregation, Instrument, MeterProviderBuilder, MetricError, Stream, new_view,
};

const ENFORCE_EXEC_TIME: &str = "kustos.enforce_execution_time_seconds";
const LOAD_POLICY_EXEC_TIME: &str = "kustos.load_policy_execution_time_seconds";
pub struct KustosMetrics {
    pub enforce_execution_time: Histogram<f64>,
    pub load_policy_execution_time: Histogram<f64>,
}

impl KustosMetrics {
    pub fn append_views(
        provider_builder: MeterProviderBuilder,
    ) -> Result<MeterProviderBuilder, MetricError> {
        Ok(provider_builder
            .with_view(new_view(
                Instrument::new().name(ENFORCE_EXEC_TIME),
                Stream::new().aggregation(Aggregation::ExplicitBucketHistogram {
                    boundaries: vec![0.01, 0.05, 0.1, 0.25, 0.5],
                    record_min_max: false,
                }),
            )?)
            .with_view(new_view(
                Instrument::new().name(LOAD_POLICY_EXEC_TIME),
                Stream::new().aggregation(Aggregation::ExplicitBucketHistogram {
                    boundaries: vec![0.01, 0.05, 0.1, 0.25, 0.5],
                    record_min_max: false,
                }),
            )?))
    }

    pub fn new(meter: &Meter) -> Self {
        Self {
            enforce_execution_time: meter
                .f64_histogram(ENFORCE_EXEC_TIME)
                .with_description("Execution time of kustos enforce")
                .with_unit("seconds")
                .build(),
            load_policy_execution_time: meter
                .f64_histogram(LOAD_POLICY_EXEC_TIME)
                .with_description("Execution time of kustos load_policy")
                .with_unit("seconds")
                .build(),
        }
    }
}
