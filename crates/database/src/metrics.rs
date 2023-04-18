// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use core::future::Future;
use core::pin::Pin;
use core::task::{ready, Poll};
use diesel::query_builder::{AsQuery, QueryFragment, QueryId};
use diesel::result::{ConnectionResult, QueryResult};
use diesel_async::pooled_connection::deadpool::Object;
use diesel_async::{
    AnsiTransactionManager, AsyncConnection, AsyncPgConnection, SimpleAsyncConnection,
    TransactionManager,
};
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::{Context, Key};
use std::sync::Arc;
use std::time::Instant;

type Parent = Object<AsyncPgConnection>;

const ERROR_KEY: Key = Key::from_static_str("error");

pub struct DatabaseMetrics {
    pub sql_execution_time: Histogram<f64>,
    pub sql_error: Counter<u64>,
    pub dbpool_connections: Histogram<u64>,
    pub dbpool_connections_idle: Histogram<i64>,
}

pub struct MetricsConnection<Conn> {
    pub(crate) metrics: Option<Arc<DatabaseMetrics>>,
    pub(crate) conn: Conn,
}

fn get_metrics_label_for_error(error: &diesel::result::Error) -> &'static str {
    match error {
        diesel::result::Error::InvalidCString(_) => "invalid_c_string",
        diesel::result::Error::DatabaseError(e, _) => match e {
            diesel::result::DatabaseErrorKind::UniqueViolation => "unique_violation",
            diesel::result::DatabaseErrorKind::ForeignKeyViolation => "foreign_key_violation",
            diesel::result::DatabaseErrorKind::UnableToSendCommand => "unable_to_send_command",
            diesel::result::DatabaseErrorKind::SerializationFailure => "serialization_failure",
            _ => "unknown",
        },
        diesel::result::Error::NotFound => unreachable!(),
        diesel::result::Error::QueryBuilderError(_) => "query_builder_error",
        diesel::result::Error::DeserializationError(_) => "deserialization_error",
        diesel::result::Error::SerializationError(_) => "serialization_error",
        diesel::result::Error::RollbackTransaction => "rollback_transaction",
        diesel::result::Error::AlreadyInTransaction => "already_in_transaction",
        _ => "unknown",
    }
}

#[async_trait::async_trait]
impl<Conn> SimpleAsyncConnection for MetricsConnection<Conn>
where
    Conn: SimpleAsyncConnection + Send,
{
    async fn batch_execute(&mut self, query: &str) -> diesel::QueryResult<()> {
        Instrument {
            metrics: self.metrics.clone(),
            future: self.conn.batch_execute(query),
            start: None,
        }
        .await
    }
}

#[async_trait::async_trait]
impl AsyncConnection for MetricsConnection<Parent> {
    type LoadFuture<'conn, 'query> =
        Instrument<BoxFuture<'query, QueryResult<Self::Stream<'conn, 'query>>>>;
    type ExecuteFuture<'conn, 'query> = Instrument<BoxFuture<'query, QueryResult<usize>>>;
    type Stream<'conn, 'query> = BoxStream<'static, QueryResult<Self::Row<'conn, 'query>>>;
    type Row<'conn, 'query> = <Parent as AsyncConnection>::Row<'conn, 'query>;
    type Backend = <Parent as AsyncConnection>::Backend;
    type TransactionManager = AnsiTransactionManager;

    async fn establish(database_url: &str) -> ConnectionResult<Self> {
        Parent::establish(database_url).await.map(|conn| Self {
            metrics: None,
            conn,
        })
    }

    #[doc(hidden)]
    fn load<'conn, 'query, T>(&'conn mut self, source: T) -> Self::LoadFuture<'conn, 'query>
    where
        T: AsQuery + Send + 'query,
        T::Query: QueryFragment<Self::Backend> + QueryId + Send + 'query,
    {
        Instrument {
            metrics: self.metrics.clone(),
            future: self.conn.load(source),
            start: None,
        }
    }

    fn execute_returning_count<'conn, 'query, T>(
        &'conn mut self,
        source: T,
    ) -> Self::ExecuteFuture<'conn, 'query>
    where
        T: QueryFragment<Self::Backend> + QueryId + Send + 'query,
    {
        Instrument {
            metrics: self.metrics.clone(),
            future: self.conn.execute_returning_count(source),
            start: None,
        }
    }

    /// Get access to the current transaction state of this connection
    ///
    /// Hidden in `diesel` behind the
    /// `i-implement-a-third-party-backend-and-opt-into-breaking-changes` feature flag,
    /// therefore not generally visible in the `diesel` generated docs.
    fn transaction_state(
        &mut self,
    ) -> &mut <Self::TransactionManager as TransactionManager<Self>>::TransactionStateData {
        self.conn.transaction_state()
    }
}

pin_project_lite::pin_project! {
    pub struct Instrument<F> {
        metrics: Option<Arc<DatabaseMetrics>>,
        #[pin]
        future: F,
        start: Option<Instant>,
    }
}

impl<F, T> Future for Instrument<F>
where
    F: Future<Output = diesel::result::QueryResult<T>>,
{
    type Output = F::Output;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();

        if let Some(metrics) = &this.metrics {
            let start = this.start.get_or_insert_with(Instant::now);

            match ready!(this.future.poll(cx)) {
                res @ (Ok(_) | Err(diesel::result::Error::NotFound)) => {
                    metrics.sql_execution_time.record(
                        &Context::current(),
                        start.elapsed().as_secs_f64(),
                        &[],
                    );

                    Poll::Ready(res)
                }
                Err(e) => {
                    let labels = &[ERROR_KEY.string(get_metrics_label_for_error(&e))];
                    metrics.sql_error.add(&Context::current(), 1, labels);

                    Poll::Ready(Err(e))
                }
            }
        } else {
            this.future.poll(cx)
        }
    }
}
