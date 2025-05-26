// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! TURN related API structs and Endpoints
#![allow(deprecated)]

use actix_web::get;

use crate::api::v1::response::NoContent;

/// Deprecated endpoint, only available for backwards compatibility.
///
/// This endpoint is deprecated and will be removed in the future.
/// It returns an empty answer regardless any configuration.
#[utoipa::path(
    operation_id = "get_turn",
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "No TURN servers have been configured",
        ),
    ),
)]
#[get("/turn")]
#[deprecated = "This endpoint and related turn settings will be removed in the future"]
pub async fn get() -> NoContent {
    NoContent {}
}
