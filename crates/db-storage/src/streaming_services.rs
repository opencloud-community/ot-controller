// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::schema::streaming_services;
use diesel::{ExpressionMethods, Identifiable, QueryDsl, Queryable};
use diesel_async::RunQueryDsl;
use opentalk_database::{DbConnection, Result};
use opentalk_types::core::{StreamingKind, StreamingServiceId};

#[derive(Debug, Queryable, Identifiable, Insertable)]
#[diesel(table_name = streaming_services)]
pub struct StreamingServiceRecord {
    pub id: StreamingServiceId,
    pub name: String,
    pub kind: StreamingKind,
    pub streaming_url: Option<String>,
    pub streaming_key_regex: Option<String>,
    pub public_url_regex: Option<String>,
}

impl StreamingServiceRecord {
    /// Retrieve a single streaming service
    #[tracing::instrument(err, skip_all)]
    pub async fn get(
        conn: &mut DbConnection,
        id: StreamingServiceId,
    ) -> Result<StreamingServiceRecord> {
        let streaming_service = streaming_services::table
            .filter(streaming_services::id.eq(id))
            .first(conn)
            .await?;

        Ok(streaming_service)
    }

    /// Retrieve all streaming services
    #[tracing::instrument(err, skip_all)]
    pub async fn get_all(conn: &mut DbConnection) -> Result<Vec<StreamingServiceRecord>> {
        let streaming_services = streaming_services::table.load(conn).await?;

        Ok(streaming_services)
    }

    /// Delete a streaming service using the given streaming service id
    #[tracing::instrument(err, skip_all)]
    pub async fn delete_by_id(
        conn: &mut DbConnection,
        streaming_service_id: StreamingServiceId,
    ) -> Result<()> {
        let _ = diesel::delete(
            streaming_services::table.filter(streaming_services::id.eq(streaming_service_id)),
        )
        .execute(conn)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = streaming_services)]
pub struct NewStreamingService {
    pub name: String,
    pub kind: StreamingKind,
    pub streaming_url: Option<String>,
    pub streaming_key_regex: Option<String>,
    pub public_url_regex: Option<String>,
}

impl NewStreamingService {
    #[tracing::instrument(err, skip_all)]
    pub async fn insert(self, conn: &mut DbConnection) -> Result<StreamingServiceRecord> {
        let query = diesel::insert_into(streaming_services::table).values(self);

        let streaming_service: StreamingServiceRecord = query.get_result(conn).await?;

        Ok(streaming_service)
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = streaming_services)]
pub struct UpdateStreamingService {
    pub name: Option<String>,
    pub kind: Option<StreamingKind>,
    pub streaming_url: Option<Option<String>>,
    pub streaming_key_regex: Option<Option<String>>,
    pub public_url_regex: Option<Option<String>>,
}

impl UpdateStreamingService {
    #[tracing::instrument(err, skip_all)]
    pub async fn apply(
        self,
        conn: &mut DbConnection,
        streaming_service_id: StreamingServiceId,
    ) -> Result<StreamingServiceRecord> {
        let query = diesel::update(streaming_services::table)
            .filter(streaming_services::id.eq(streaming_service_id))
            .set(self)
            .returning(streaming_services::all_columns);

        let invite = query.get_result(conn).await?;

        Ok(invite)
    }
}
