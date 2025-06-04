// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{sync::Arc, time::Instant};

use opentelemetry::{
    metrics::{Histogram, Meter},
    Key, KeyValue,
};
use opentelemetry_sdk::metrics::{
    new_view, Aggregation, Instrument, MeterProviderBuilder, MetricError, Stream,
};
use redis::{aio::ConnectionLike, Arg, RedisFuture};

const COMMAND_KEY: Key = Key::from_static_str("command");
const EXEC_TIME: &str = "redis.command_execution_time_seconds";
pub struct RedisMetrics {
    pub command_execution_time: Histogram<f64>,
}

impl RedisMetrics {
    pub fn append_views(
        provider_builder: MeterProviderBuilder,
    ) -> Result<MeterProviderBuilder, MetricError> {
        Ok(provider_builder.with_view(new_view(
            Instrument::new().name(EXEC_TIME),
            Stream::new().aggregation(Aggregation::ExplicitBucketHistogram {
                boundaries: vec![0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5],
                record_min_max: false,
            }),
        )?))
    }

    pub fn new(meter: &Meter) -> Self {
        Self {
            command_execution_time: meter
                .f64_histogram(EXEC_TIME)
                .with_description("Execution time of redis commands in seconds")
                .with_unit("seconds")
                .build(),
        }
    }
}

#[derive(Clone)]
pub struct RedisConnection {
    connection_manager: redis::aio::ConnectionManager,
    metrics: Option<Arc<RedisMetrics>>,
}

impl RedisConnection {
    pub fn new(connection_manager: redis::aio::ConnectionManager) -> Self {
        Self {
            connection_manager,
            metrics: None,
        }
    }

    pub fn with_metrics(mut self, metrics: Arc<RedisMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub fn into_manager(self) -> redis::aio::ConnectionManager {
        self.connection_manager
    }
}

impl ConnectionLike for RedisConnection {
    fn req_packed_command<'a>(&'a mut self, cmd: &'a redis::Cmd) -> RedisFuture<'a, redis::Value> {
        let fut = self.connection_manager.req_packed_command(cmd);

        if let Some(metrics) = &self.metrics {
            Box::pin(async move {
                let start = Instant::now();

                let res = fut.await;

                if res.is_ok() {
                    let command = if let Some(Arg::Simple(b)) = cmd.args_iter().next() {
                        KeyValue::new(
                            COMMAND_KEY,
                            std::str::from_utf8(b).unwrap_or("UNKNOWN").to_owned(),
                        )
                    } else {
                        KeyValue::new(COMMAND_KEY, "UNKNOWN")
                    };

                    metrics
                        .command_execution_time
                        .record(start.elapsed().as_secs_f64(), &[command]);
                }

                res
            })
        } else {
            fut
        }
    }

    fn req_packed_commands<'a>(
        &'a mut self,
        cmd: &'a redis::Pipeline,
        offset: usize,
        count: usize,
    ) -> RedisFuture<'a, Vec<redis::Value>> {
        let fut = self
            .connection_manager
            .req_packed_commands(cmd, offset, count);

        if let Some(metrics) = &self.metrics {
            Box::pin(async move {
                let start = Instant::now();

                let res = fut.await;

                if res.is_ok() {
                    metrics.command_execution_time.record(
                        start.elapsed().as_secs_f64(),
                        &[KeyValue::new(COMMAND_KEY, "MULTI")],
                    );
                }

                res
            })
        } else {
            fut
        }
    }

    fn get_db(&self) -> i64 {
        self.connection_manager.get_db()
    }
}
