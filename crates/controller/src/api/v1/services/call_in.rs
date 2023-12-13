// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::api::signaling::ticket::start_or_continue_signaling_session;
use crate::api::v1::response::ApiError;
use crate::api::v1::util::require_feature;
use crate::settings::SharedSettingsActix;
use actix_web::dev::HttpServiceFactory;
use actix_web::error::Result;
use actix_web::post;
use actix_web::web::{Data, Json};
use database::Db;
use db_storage::sip_configs::SipConfig;
use signaling_core::{Participant, RedisConnection};
use types::api::v1::services::{ServiceStartResponse, StartRequestBody};
use types::common::features;
use validator::Validate;

// Note to devs:
// Please update `docs/admin/keycloak.md` service login documentation as well if
// you change something here
pub const REQUIRED_CALL_IN_ROLE: &str = "opentalk-call-in";

/// API Endpoint *POST services/call_in/start* for the call-in service
#[post("/start")]
pub async fn start(
    settings: SharedSettingsActix,
    db: Data<Db>,
    redis_ctx: Data<RedisConnection>,
    request: Json<StartRequestBody>,
) -> Result<Json<ServiceStartResponse>, ApiError> {
    let settings = settings.load();
    let mut redis_conn = (**redis_ctx).clone();
    let request = request.into_inner();

    let mut conn = db.get_conn().await?;

    let (sip_config, room) = SipConfig::get_with_room(&mut conn, &request.id)
        .await?
        .ok_or_else(invalid_credentials_error)?;

    require_feature(&mut conn, &settings, room.created_by, features::CALL_IN).await?;

    request.id.validate()?;
    request.pin.validate()?;

    if sip_config.password != request.pin {
        return Err(invalid_credentials_error());
    }

    drop(conn);

    let (ticket, resumption) =
        start_or_continue_signaling_session(&mut redis_conn, Participant::Sip, room.id, None, None)
            .await?;

    Ok(Json(ServiceStartResponse { ticket, resumption }))
}

fn invalid_credentials_error() -> ApiError {
    ApiError::bad_request()
        .with_code("invalid_credentials")
        .with_message("given call-in id & pin combination is not valid")
}

pub fn services() -> impl HttpServiceFactory {
    actix_web::web::scope("/call_in")
        .wrap(super::RequiredRealmRole::new(REQUIRED_CALL_IN_ROLE))
        .service(start)
}
