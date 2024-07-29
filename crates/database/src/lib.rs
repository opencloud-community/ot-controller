// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! OpenTalk Database connector, interface and connection handling

use diesel::{
    pg::Pg,
    query_builder::{AstPass, Query, QueryFragment, QueryId},
    sql_types::BigInt,
    QueryResult,
};
use diesel_async::{
    methods::LoadQuery,
    pooled_connection::deadpool::{BuildError, Object, PoolError},
    AsyncConnection, AsyncPgConnection,
};
use snafu::Snafu;

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
#[derive(Debug, Snafu)]
pub enum DatabaseError {
    #[snafu(display("Database Error: `{message}`",))]
    Custom { message: String },

    #[snafu(display("Diesel Error: `{source}`",))]
    DieselError { source: diesel::result::Error },

    #[snafu(display("A requested resource could not be found"))]
    NotFound,

    #[snafu(display("Deadpool build error: `{source}`",), context(false))]
    DeadpoolBuildError { source: BuildError },

    #[snafu(display("Deadpool error: `{source}`",))]
    DeadpoolError { source: PoolError },

    #[snafu(context(false))]
    UrlParseError { source: url::ParseError },
}

impl DatabaseError {
    /// Returns `true` if the database error is [`NotFound`].
    ///
    /// [`NotFound`]: DatabaseError::NotFound
    #[must_use]
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound)
    }
}

impl From<diesel::result::Error> for DatabaseError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::NotFound => Self::NotFound,
            source => Self::DieselError { source },
        }
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
        let total = results.first().map(|x: &(U, i64)| x.1).unwrap_or(0);
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

#[cfg(test)]
mod tests {
    use crate::DatabaseError;

    #[test]
    fn test_database_error_from_implementation() {
        // The `diesel::result::Error::NotFound` should also be a
        // `DatabaseError::NotFound` and never be a `DatabaseError::DieselError`
        // All other cases should be `DatabaseError::DieselError`
        assert!(matches!(
            Into::<DatabaseError>::into(diesel::result::Error::NotFound),
            DatabaseError::NotFound,
        ));
        assert!(!matches!(
            Into::<DatabaseError>::into(diesel::result::Error::NotFound),
            DatabaseError::DieselError {
                source: diesel::result::Error::NotFound
            },
        ));
        assert!(matches!(
            Into::<DatabaseError>::into(diesel::result::Error::NotInTransaction),
            DatabaseError::DieselError {
                source: diesel::result::Error::NotInTransaction
            },
        ));
    }
}
