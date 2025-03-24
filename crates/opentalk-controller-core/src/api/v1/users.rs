// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! User related API structs and Endpoints
//!
//! The defined structs are exposed to the REST API and will be serialized/deserialized. Similar
//! structs are defined in the Database crate [`opentalk_db_storage`] for database operations.

use actix_web::{
    get, patch,
    web::{Data, Json, Path, Query, ReqData},
    Either,
};
use chrono::Utc;
use openidconnect::AccessToken;
use opentalk_controller_service::oidc::{decode_token, UserClaims};
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_controller_utils::CaptureApiError;
use opentalk_database::Db;
use opentalk_db_storage::{tenants::Tenant, users::User};
use opentalk_types_api_v1::{
    assets::AssetSortingQuery,
    error::ApiError,
    pagination::PagePaginationQuery,
    rooms::RoomResource,
    users::{
        me::PatchMeRequestBody, GetFindQuery, GetFindResponseBody, GetUserAssetsResponseBody,
        PrivateUserProfile, PublicUserProfile,
    },
};
use opentalk_types_common::{tariffs::TariffResource, tenants::TenantId, users::UserId};
use snafu::{Report, ResultExt, Whatever};

use super::response::NoContent;
use crate::{
    api::{
        responses::{Forbidden, InternalServerError, Unauthorized},
        v1::ApiResponse,
    },
    caches::Caches,
};

/// Patch the current user's profile
///
/// Fields that are not provided in the request body will remain unchanged.
#[utoipa::path(
    request_body = PatchMeRequestBody,
    operation_id = "patch_users_me",
    responses(
        (
            status = StatusCode::OK,
            description = "User profile was successfully updated",
            body = RoomResource
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = r"Could not modify the user's profile due to wrong
                syntax or bad values",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[patch("/users/me")]
pub async fn patch_me(
    service: Data<OpenTalkControllerService>,
    db: Data<Db>,
    caches: Data<Caches>,
    access_token: ReqData<AccessToken>,
    current_user: ReqData<RequestUser>,
    patch: Json<PatchMeRequestBody>,
) -> Result<Either<Json<PrivateUserProfile>, NoContent>, ApiError> {
    let current_user = current_user.into_inner();

    let user_profile = service
        .patch_me(current_user.clone(), patch.into_inner())
        .await?;

    // Update the middleware's cached items as well to reflect the changes immediately.
    update_middleware_cache(
        &db,
        &caches,
        current_user.id,
        current_user.tenant_id,
        access_token.into_inner(),
    )
    .await?;

    match user_profile {
        Some(user_profile) => Ok(Either::Left(Json(user_profile))),
        _ => Ok(Either::Right(NoContent)),
    }
}

async fn update_middleware_cache(
    db: &Db,
    caches: &Caches,
    user_id: UserId,
    tenant_id: TenantId,
    access_token: AccessToken,
) -> Result<(), CaptureApiError> {
    let mut conn = db.get_conn().await?;
    let user = User::get(&mut conn, user_id).await?;
    let tenant = Tenant::get(&mut conn, tenant_id).await?;

    let user_claims = decode_token::<UserClaims>(access_token.secret())
        .whatever_context::<&str, Whatever>(
            "failed to decode access token for user profile update",
        )?;

    let token_ttl = user_claims.exp - Utc::now();
    if token_ttl > chrono::Duration::seconds(10) {
        match token_ttl.to_std() {
            Ok(token_ttl_std) => {
                caches
                    .user_access_tokens
                    .insert_with_ttl(
                        access_token.secret().clone(),
                        Ok((tenant, user)),
                        token_ttl_std,
                    )
                    .await?;
            }
            Err(e) => {
                log::debug!(
                    "abort user profile cache update due to invalid token TTL, {}",
                    Report::from_error(e)
                );
            }
        }
    }
    Ok(())
}

/// Get the current user's profile
///
/// Returns the private user profile of the currently logged-in user. This
/// private profile contains information that is not visible in the public
/// profile, such as tariff status or the used storage.
#[utoipa::path(
    responses(
        (
            status = StatusCode::OK,
            description = "Information about the logged in user",
            body = PrivateUserProfile,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[get("/users/me")]
pub async fn get_me(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
) -> Result<Json<PrivateUserProfile>, ApiError> {
    Ok(Json(service.get_me(current_user.into_inner()).await?))
}

/// Get the current user tariff information.
///
/// Returns the tariff information for the currently logged in user.
#[utoipa::path(
    responses(
        (
            status = StatusCode::OK,
            description = "Information about the tariff of the current user",
            body = TariffResource,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[get("/users/me/tariff")]
pub async fn get_me_tariff(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
) -> Result<Json<TariffResource>, ApiError> {
    let resource = service.get_my_tariff(current_user.into_inner()).await?;
    Ok(Json(resource))
}

/// Get the assets associated with the user.
///
/// All assets associated to the requesting user are returned in a list. If no
/// pagination query is added, the default page size is used.
#[utoipa::path(
    params(PagePaginationQuery, AssetSortingQuery),
    responses(
        (
            status = StatusCode::OK,
            description = "List of accessible assets successfully returned",
            body = GetUserAssetsResponseBody,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[get("/users/me/assets")]
pub async fn get_me_assets(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    sorting: Query<AssetSortingQuery>,
    pagination: Query<PagePaginationQuery>,
) -> Result<ApiResponse<GetUserAssetsResponseBody>, ApiError> {
    let (assets_response, asset_count) = service
        .get_my_assets(current_user.into_inner(), sorting.into_inner(), &pagination)
        .await?;

    Ok(ApiResponse::new(assets_response).with_page_pagination(
        pagination.per_page,
        pagination.page,
        asset_count,
    ))
}

/// Get a user's public profile
///
/// Returns the public profile of a user.
#[utoipa::path(
    params(
        ("user_id" = UserId, description = "The id of the user"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "Information about the user",
            body = PublicUserProfile,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::FORBIDDEN,
            response = Forbidden,
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[get("/users/{user_id}")]
pub async fn get_user(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    user_id: Path<UserId>,
) -> Result<Json<PublicUserProfile>, ApiError> {
    let user_profile = Json(
        service
            .get_user(current_user.into_inner(), user_id.into_inner())
            .await?,
    );
    Ok(user_profile)
}

/// Find users
///
/// Query users for autocomplete fields
#[utoipa::path(
    params(GetFindQuery),
    responses(
        (
            status = StatusCode::OK,
            description = "Search results",
            body = GetFindResponseBody,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[get("/users/find")]
pub async fn find(
    service: Data<OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    query: Query<GetFindQuery>,
) -> Result<Json<GetFindResponseBody>, ApiError> {
    let result = Json(
        service
            .find_users(current_user.into_inner(), query.into_inner())
            .await?,
    );
    Ok(result)
}
