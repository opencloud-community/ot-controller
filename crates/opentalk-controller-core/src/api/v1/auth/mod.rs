// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Auth related API structs and Endpoints

#![allow(deprecated)]

use actix_web::{
    get, post,
    web::{Data, Json},
};
use opentalk_controller_service::oidc::{OidcContext, VerifyError};
use opentalk_controller_service_facade::OpenTalkControllerService;
use opentalk_controller_utils::CaptureApiError;
use opentalk_types_api_v1::{
    auth::{GetLoginResponseBody, PostLoginResponseBody, login::AuthLoginPostRequestBody},
    error::{ApiError, AuthenticationError, ErrorBody},
};

use crate::api::responses::InternalServerError;

/// **Deprecated**: This endpoint exists only for backwards compatibility and must no longer be used.
///
/// The login endpoint
///
/// Attempt to authenticate with a provided ID token. The ID token can be
/// received from an OIDC provider and contains information about the requesting
/// user as well as an expiration timestamp. When a valid token with an unknown user
/// is provided, a new user will be created in the database.
#[utoipa::path(
    request_body = AuthLoginPostRequestBody,
    responses(
        (
            status = StatusCode::OK,
            description = "Login successful, answer contains a list of permissions",
            body = PostLoginResponseBody,
            example = json!({"permissions": []})
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = "The provided ID token is malformed or contains invalid claims",
            body = ErrorBody,
            example = json!(
                ApiError::bad_request()
                    .with_code("invalid_claims")
                    .with_message("some required attributes are missing or malformed")
                    .body
            ),
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "The provided ID token is invalid",
            body = ErrorBody,
            example = json!(
                ApiError::unauthorized().with_www_authenticate(AuthenticationError::InvalidIdToken).body
            ),
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(),
)]
#[post("/auth/login")]
#[deprecated]
pub async fn post_login(
    oidc_ctx: Data<OidcContext>,
    body: Json<AuthLoginPostRequestBody>,
) -> Result<Json<PostLoginResponseBody>, ApiError> {
    Ok(post_login_inner(&oidc_ctx, body.into_inner().id_token).await?)
}

async fn post_login_inner(
    oidc_ctx: &OidcContext,
    id_token: String,
) -> Result<Json<PostLoginResponseBody>, CaptureApiError> {
    if let Err(e) = oidc_ctx.verify_id_token(&id_token) {
        return match e {
            VerifyError::InvalidClaims => Err(ApiError::bad_request()
                .with_code("invalid_claims")
                .with_message("some required attributes are missing or malformed")
                .into()),
            VerifyError::Expired { .. } => Err(ApiError::unauthorized()
                .with_www_authenticate(AuthenticationError::SessionExpired)
                .into()),
            VerifyError::MissingKeyID
            | VerifyError::UnknownKeyID
            | VerifyError::MalformedSignature
            | VerifyError::InvalidJwt { .. }
            | VerifyError::InvalidSignature => Err(ApiError::unauthorized()
                .with_www_authenticate(AuthenticationError::InvalidIdToken)
                .into()),
        };
    };

    Ok(Json(PostLoginResponseBody {
        // TODO calculate permissions
        permissions: Default::default(),
    }))
}

/// Get the configured OIDC provider
///
/// Returns the relevant information for a frontend to authenticate against the
/// configured OIDC provider for the OpenTalk service.
#[utoipa::path(
    responses(
        (
            status = StatusCode::OK,
            description = "Get information about the OIDC provider",
            body = GetLoginResponseBody,
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(),
)]
#[get("/auth/login")]
pub async fn get_login(service: Data<OpenTalkControllerService>) -> Json<GetLoginResponseBody> {
    Json(service.get_login().await)
}
