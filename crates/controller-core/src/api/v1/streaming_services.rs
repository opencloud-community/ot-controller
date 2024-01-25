// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! streaming service related API structs and Endpoints
use super::DefaultApiResult;
use crate::api::v1::ApiResponse;
use actix_web::{
    get,
    web::{Data, Path},
};
use anyhow::Context;
use opentalk_database::{Db, DbConnection};
use opentalk_db_storage::streaming_services::StreamingServiceRecord;
use opentalk_types::{
    api::{
        error::ApiError,
        v1::streaming_services::{
            GetStreamingServiceResponse, GetStreamingServicesResponse, StreamingServiceIdentifier,
        },
    },
    common::streaming::{StreamingService, StreamingServiceKind},
    core::StreamingKind,
};

/// API Endpoint *GET /users/me/streaming_services*
///
/// Returns a JSON array of all streaming services
#[get("/users/me/streaming_services")]
pub async fn get_me_streaming_services(
    db: Data<Db>,
) -> DefaultApiResult<GetStreamingServicesResponse> {
    let mut conn = db.get_conn().await?;

    let streaming_services = get_streaming_services(&mut conn).await?;

    Ok(ApiResponse::new(GetStreamingServicesResponse(
        streaming_services,
    )))
}

pub(super) async fn get_streaming_services(
    conn: &mut DbConnection,
) -> Result<Vec<StreamingService>, ApiError> {
    let streaming_services = StreamingServiceRecord::get_all(conn).await?;

    let streaming_services = streaming_services
        .into_iter()
        .map(|streaming_service_record| {
            let streaming_service = create_streaming_service_from_record(streaming_service_record)?;

            Ok(streaming_service)
        })
        .collect::<Result<Vec<_>, ApiError>>()?;

    Ok(streaming_services)
}

/// API Endpoint *GET /users/me/streaming_services/{streaming_service_id}*
///
/// Returns a single streaming service.
/// Returns 401 Not Found when the user has no access.
#[get("/users/me/streaming_services/{streaming_service_id}")]
pub async fn get_me_streaming_service(
    db: Data<Db>,
    path_params: Path<StreamingServiceIdentifier>,
) -> DefaultApiResult<GetStreamingServiceResponse> {
    let mut conn = db.get_conn().await?;
    let StreamingServiceIdentifier {
        streaming_service_id,
    } = path_params.into_inner();

    let streaming_service_record =
        StreamingServiceRecord::get(&mut conn, streaming_service_id).await?;

    let streaming_service = create_streaming_service_from_record(streaming_service_record)?;

    Ok(ApiResponse::new(GetStreamingServiceResponse(
        streaming_service,
    )))
}

fn create_streaming_service_from_record(
    streaming_service_record: StreamingServiceRecord,
) -> Result<StreamingService, ApiError> {
    let kind = match streaming_service_record.kind {
        StreamingKind::Builtin => StreamingServiceKind::Builtin,
        StreamingKind::Custom => StreamingServiceKind::Custom,
        StreamingKind::Provider => StreamingServiceKind::Provider,
    };

    let streaming_url = if let Some(streaming_url) = streaming_service_record.streaming_url {
        Ok(streaming_url
            .parse()
            .context("invalid streaming endpoint url entry in db")?)
    } else {
        None
    };

    let streaming_service = StreamingService {
        id: streaming_service_record.id,
        name: streaming_service_record.name,
        kind,
        streaming_url,
        streaming_key_regex: streaming_service_record.streaming_key_regex,
        public_url_regex: streaming_service_record.public_url_regex,
    };

    Ok(streaming_service)
}
