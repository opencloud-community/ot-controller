// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::metrics::{DatabaseMetrics, MetricsConnection};
use crate::{DatabaseError, DbConnection};
use controller_settings as settings;
use deadpool_runtime::Runtime;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use opentelemetry::Context;
use std::sync::Arc;
use std::time::Duration;

type DbPool = Pool<AsyncPgConnection>;

/// Db container that uses a connection pool to hand out connections
///
/// Uses an deadpool connection pool to manage multiple established connections.
pub struct Db {
    metrics: Option<Arc<DatabaseMetrics>>,
    pool: DbPool,
}

impl Db {
    /// Creates a new Db instance from the specified database settings.
    #[tracing::instrument(skip(db_settings))]
    pub fn connect(db_settings: &settings::Database) -> crate::Result<Self> {
        Self::connect_url(&db_settings.url, db_settings.max_connections)
    }

    /// Creates a new Db instance from the specified database url.
    pub fn connect_url(db_url: &str, max_conns: u32) -> crate::Result<Self> {
        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(db_url);

        let pool = Pool::builder(manager)
            .max_size(max_conns as usize)
            .create_timeout(Some(Duration::from_secs(10)))
            .runtime(Runtime::Tokio1)
            .build()?;

        Ok(Self {
            metrics: None,
            pool,
        })
    }

    /// Set the metrics to use for this database pool
    pub fn set_metrics(&mut self, metrics: Arc<DatabaseMetrics>) {
        self.metrics = Some(metrics);
    }

    /// Returns an established connection from the connection pool
    #[tracing::instrument(skip_all)]
    pub async fn get_conn(&self) -> crate::Result<DbConnection> {
        let res = self.pool.get().await;
        let state = self.pool.status();

        if let Some(metrics) = &self.metrics {
            let context = Context::current();
            metrics
                .dbpool_connections
                .record(&context, state.size as u64, &[]);

            metrics
                .dbpool_connections_idle
                .record(&context, state.available as i64, &[]);
        }

        match res {
            Ok(conn) => {
                let conn = MetricsConnection {
                    metrics: self.metrics.clone(),
                    conn,
                };

                Ok(conn)
            }
            Err(e) => {
                let state = self.pool.status();
                let msg = format!(
                    "Unable to get connection from connection pool.
                                Error: {e}
                                Pool State:
                                    {state:?}",
                );
                log::error!("{}", &msg);
                Err(DatabaseError::DeadpoolError(e))
            }
        }
    }
}
