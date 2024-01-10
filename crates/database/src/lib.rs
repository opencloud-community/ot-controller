// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! OpenTalk Database connector, interface and connection handling

use diesel::pg::Pg;
use diesel::query_builder::{AstPass, Query, QueryFragment, QueryId};
use diesel::result::Error;
use diesel::sql_types::BigInt;
use diesel::QueryResult;
use diesel_async::methods::LoadQuery;
use diesel_async::pooled_connection::deadpool::{BuildError, Object, PoolError};
use diesel_async::{AsyncConnection, AsyncPgConnection};
use std::borrow::Cow;

mod db;
mod metrics;
pub mod query_helper;

pub use db::Db;
pub use metrics::DatabaseMetrics;

/// Pooled connection alias
pub type DbConnection = metrics::MetricsConnection<Object<AsyncPgConnection>>;

/// Result type using [`DatabaseError`] as a default Error
pub type Result<T, E = DatabaseError> = std::result::Result<T, E>;

/// Error types for the database abstraction
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Database Error: `{0}`")]
    Custom(Cow<'static, str>),
    #[error("Diesel Error: `{0}`")]
    DieselError(diesel::result::Error),
    #[error("A requested resource could not be found")]
    NotFound,
    #[error("Deadpool build error: `{0}`")]
    DeadpoolBuildError(#[from] BuildError),
    #[error("Deadpool error: `{0}`")]
    DeadpoolError(#[from] PoolError),
}

impl DatabaseError {
    pub fn custom<I>(error_string: I) -> Self
    where
        I: Into<Cow<'static, str>>,
    {
        Self::Custom(error_string.into())
    }
}

pub trait OptionalExt<T, E> {
    fn optional(self) -> Result<Option<T>, E>;
}

impl<T> OptionalExt<T, DatabaseError> for Result<T, DatabaseError> {
    fn optional(self) -> Result<Option<T>, DatabaseError> {
        match self {
            Ok(t) => Ok(Some(t)),
            Err(DatabaseError::NotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

impl From<diesel::result::Error> for DatabaseError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            Error::NotFound => Self::NotFound,
            err => DatabaseError::DieselError(err),
        }
    }
}
/// Pagination trait for diesel
pub trait Paginate: Sized {
    fn paginate(self, page: i64) -> Paginated<Self>;
    fn paginate_by(self, per_page: i64, page: i64) -> Paginated<Self>;
}

impl<T> Paginate for T {
    fn paginate(self, page: i64) -> Paginated<Self> {
        Paginated {
            query: self,
            per_page: DEFAULT_PER_PAGE,
            offset: (page - 1) * DEFAULT_PER_PAGE,
        }
    }
    fn paginate_by(self, per_page: i64, page: i64) -> Paginated<Self> {
        Paginated {
            query: self,
            per_page,
            offset: (page - 1) * per_page,
        }
    }
}

const DEFAULT_PER_PAGE: i64 = 10;

/// Paginated diesel database response
#[derive(Debug, Clone, Copy, QueryId)]
pub struct Paginated<T> {
    query: T,
    per_page: i64,
    // We need to store the offset instead of the page due to
    // lifetime requirements in `QueryFragment::walk_ast(...)`.
    offset: i64,
}

impl<T> Paginated<T> {
    pub fn per_page(self, per_page: i64) -> Self {
        Paginated { per_page, ..self }
    }

    pub async fn load_and_count<'query, U, Conn>(
        self,
        conn: &mut Conn,
    ) -> QueryResult<(Vec<U>, i64)>
    where
        Self: LoadQuery<'query, Conn, (U, i64)>,
        Conn: AsyncConnection,
        U: Send + 'static,
        T: 'query,
    {
        let results: Vec<(U, i64)> = {
            // When `diesel_async::RunQueryDsl` is imported globally, the call
            // to `results.first()` below will cause compiler errors because the
            // compiler mistakes it for `diesel_async::RunQueryDsl::first(…)`
            // and fails finding a trait implementation of `results` that
            // matches, so we restrict the import scope.
            use diesel_async::RunQueryDsl;
            self.load::<(U, i64)>(conn).await?
        };
        let total = results.first().map(|x| x.1).unwrap_or(0);
        let records = results.into_iter().map(|x| x.0).collect();
        Ok((records, total))
    }
}

impl<T: Query> Query for Paginated<T> {
    type SqlType = (T::SqlType, BigInt);
}

impl<T> QueryFragment<Pg> for Paginated<T>
where
    T: QueryFragment<Pg>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> QueryResult<()> {
        out.push_sql("SELECT *, COUNT(*) OVER () FROM (");
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(") t LIMIT ");
        out.push_bind_param::<BigInt, _>(&self.per_page)?;
        out.push_sql(" OFFSET ");
        out.push_bind_param::<BigInt, _>(&self.offset)?;
        Ok(())
    }
}
