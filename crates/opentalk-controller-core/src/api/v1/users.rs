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
use opentalk_controller_settings::{TenantAssignment, UsersFindBehavior};
use opentalk_database::Db;
use opentalk_db_storage::{
    assets,
    tariffs::Tariff,
    tenants::Tenant,
    users::{email_to_libravatar_url, UpdateUser, User},
};
use opentalk_keycloak_admin::KeycloakAdminClient;
use opentalk_types::api::{
    error::ApiError,
    v1::{
        order::{AssetSorting, SortingQuery},
        users::PatchMeBody,
    },
};
use opentalk_types_api_v1::{
    pagination::PagePaginationQuery,
    users::{
        GetFindQuery, GetFindResponseBody, GetFindResponseEntry, GetUserAssetsResponseBody,
        PrivateUserProfile, PublicUserProfile, UnregisteredUser,
    },
};
use opentalk_types_common::{tariffs::TariffResource, users::UserId};
use snafu::{Report, ResultExt, Whatever};
use validator::Validate;

use super::response::NoContent;
use crate::{
    api::{
        responses::{Forbidden, InternalServerError, Unauthorized},
        signaling::SignalingModules,
        v1::ApiResponse,
    },
    caches::Caches,
    oidc::{decode_token, UserClaims},
    settings::SharedSettingsActix,
};

const MAX_USER_SEARCH_RESULTS: usize = 20;

/// Patch the current user's profile
///
/// Fields that are not provided in the request body will remain unchanged.
#[utoipa::path(
    request_body = PatchMeBody,
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
    settings: SharedSettingsActix,
    db: Data<Db>,
    caches: Data<Caches>,
    current_user: ReqData<User>,
    current_tenant: ReqData<Tenant>,
    access_token: ReqData<AccessToken>,
    patch: Json<PatchMeBody>,
) -> Result<Either<Json<PrivateUserProfile>, NoContent>, ApiError> {
    let patch = patch.into_inner();

    if patch.is_empty() {
        return Ok(Either::Right(NoContent));
    }

    patch.validate()?;

    let settings = settings.load_full();

    let mut conn = db.get_conn().await?;

    // Prohibit display name editing, if configured
    if settings.endpoints.disallow_custom_display_name {
        if let Some(display_name) = &patch.display_name {
            if &current_user.display_name != display_name {
                return Err(
                    ApiError::bad_request().with_message("changing the display name is prohibited")
                );
            }
        }
    }

    let changeset = UpdateUser {
        title: patch.title.as_ref(),
        firstname: None,
        lastname: None,
        avatar_url: None,
        phone: None,
        email: None,
        display_name: patch.display_name.as_ref(),
        language: patch.language.as_deref(),
        dashboard_theme: patch.dashboard_theme.as_deref(),
        conference_theme: patch.conference_theme.as_deref(),
        id_token_exp: None,
        tariff_id: None,
        tariff_status: None,
        disabled_since: None,
    };

    let user = changeset.apply(&mut conn, current_user.id).await?;
    let used_storage = User::get_used_storage_u64(&mut conn, &current_user.id).await?;

    let user_profile = user.to_private_user_profile(&settings, used_storage);

    let user_claims = decode_token::<UserClaims>(access_token.clone().secret())
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
                        Ok((current_tenant.into_inner(), user)),
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

    Ok(Either::Left(Json(user_profile)))
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
    settings: SharedSettingsActix,
    db: Data<Db>,
    current_user: ReqData<User>,
) -> Result<Json<PrivateUserProfile>, ApiError> {
    let settings = settings.load_full();
    let current_user = current_user.into_inner();

    let used_storage =
        User::get_used_storage_u64(&mut db.get_conn().await?, &current_user.id).await?;

    let user_profile = current_user.to_private_user_profile(&settings, used_storage);

    Ok(Json(user_profile))
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
    settings: SharedSettingsActix,
    db: Data<Db>,
    modules: Data<SignalingModules>,
    current_user: ReqData<User>,
) -> Result<Json<TariffResource>, ApiError> {
    let settings = settings.load_full();

    let current_user = current_user.into_inner();

    let mut conn = db.get_conn().await?;

    let tariff = Tariff::get(&mut conn, current_user.tariff_id).await?;

    let response = tariff.to_tariff_resource(
        settings.defaults.disabled_features.clone(),
        modules.get_module_features(),
    );

    Ok(Json(response))
}

/// Get the assets associated with the user.
///
/// All assets associated to the requesting user are returned in a list. If no
/// pagination query is added, the default page size is used.
#[utoipa::path(
    params(PagePaginationQuery, SortingQuery<AssetSorting>),
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
    db: Data<Db>,
    current_user: ReqData<User>,
    sorting: Query<SortingQuery<AssetSorting>>,
    pagination: Query<PagePaginationQuery>,
) -> Result<ApiResponse<GetUserAssetsResponseBody>, ApiError> {
    let current_user = current_user.into_inner();
    let pagination = pagination.into_inner();
    let sorting = sorting.into_inner();

    let mut conn = db.get_conn().await?;

    let (owned_assets, asset_count) = assets::get_all_for_room_owner_paginated_ordered(
        &mut conn,
        current_user.id,
        pagination.per_page,
        pagination.page,
        sorting,
    )
    .await?;

    Ok(
        ApiResponse::new(GetUserAssetsResponseBody { owned_assets }).with_page_pagination(
            pagination.per_page,
            pagination.page,
            asset_count,
        ),
    )
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
    settings: SharedSettingsActix,
    db: Data<Db>,
    current_user: ReqData<User>,
    user_id: Path<UserId>,
) -> Result<Json<PublicUserProfile>, ApiError> {
    let settings = settings.load_full();

    let mut conn = db.get_conn().await?;

    let user =
        User::get_filtered_by_tenant(&mut conn, current_user.tenant_id, user_id.into_inner())
            .await?;

    let user_profile = user.to_public_user_profile(&settings);

    Ok(Json(user_profile))
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
    settings: SharedSettingsActix,
    kc_admin_client: Data<KeycloakAdminClient>,
    db: Data<Db>,
    current_tenant: ReqData<Tenant>,
    query: Query<GetFindQuery>,
) -> Result<Json<GetFindResponseBody>, ApiError> {
    let settings = settings.load_full();

    let oidc_and_user_search_configuration =
        settings
            .build_oidc_and_user_search_configuration()
            .whatever_context::<&str, Whatever>("Failed to build user search settings")?;

    if oidc_and_user_search_configuration
        .user_search
        .users_find_behavior
        == UsersFindBehavior::Disabled
    {
        return Err(ApiError::not_found());
    }

    if query.q.len() < 3 {
        return Err(ApiError::bad_request()
            .with_code("query_too_short")
            .with_message("query must be at least 3 characters long"));
    }

    // Get all users from Keycloak matching the search criteria
    let found_users = if oidc_and_user_search_configuration
        .user_search
        .users_find_behavior
        == UsersFindBehavior::FromUserSearchBackend
    {
        let mut found_kc_users = match &settings.tenants.assignment {
            TenantAssignment::Static { .. } => {
                // Do not filter by tenant_id if the assignment is static, since that's used
                // when Keycloak does not provide any tenant information we can filter over anyway
                kc_admin_client
                    .search_user(&query.q, MAX_USER_SEARCH_RESULTS)
                    .await
                    .whatever_context::<&str, Whatever>("Failed to search for user in keycloak")?
            }
            TenantAssignment::ByExternalTenantId {
                external_tenant_id_user_attribute_name,
            } => {
                // Keycloak must contain information about the tenancy of a user,
                // so we pass in the tenant_id to filter the found users
                kc_admin_client
                    .search_user_filtered(
                        external_tenant_id_user_attribute_name,
                        current_tenant.oidc_tenant_id.as_ref(),
                        &query.q,
                        MAX_USER_SEARCH_RESULTS,
                    )
                    .await
                    .whatever_context::<&str, Whatever>("Failed to search for user in keycloak")?
            }
        };

        let mut conn = db.get_conn().await?;

        // Get the name of the user attribute providing the Keycloak user ids. If set to None,
        // Keycloak's id field is used for user assignment instead.
        let user_attribute_name = oidc_and_user_search_configuration
            .user_search
            .external_id_user_attribute_name
            .as_deref();

        // Get the Keycloak user ids for Keycloak users found above, omitting users for whom no Keycloak
        // user id is provided
        let keycloak_user_ids: Vec<&str> = found_kc_users
            .iter()
            .filter_map(|kc_user| kc_user.get_keycloak_user_id(user_attribute_name))
            .collect();

        // For all Keycloak users found above, get the according users already registered from the database
        let registered_users =
            User::get_all_by_oidc_subs(&mut conn, current_tenant.id, &keycloak_user_ids).await?;

        // From the list of Keycloak users found above, remove all users already registered
        found_kc_users.retain(|kc_user| {
            if let Some(keycloak_user_id) = kc_user.get_keycloak_user_id(user_attribute_name) {
                !registered_users
                    .iter()
                    .any(|user| user.oidc_sub == keycloak_user_id)
            } else {
                true
            }
        });

        // Build a list of registered and unregistered users resulting from the search
        registered_users
            .into_iter()
            .map(|user| GetFindResponseEntry::Registered(user.to_public_user_profile(&settings)))
            .chain(found_kc_users.into_iter().map(|kc_user| {
                let avatar_url =
                    email_to_libravatar_url(&settings.avatar.libravatar_url, &kc_user.email);

                GetFindResponseEntry::Unregistered(UnregisteredUser {
                    email: kc_user.email,
                    firstname: kc_user.first_name,
                    lastname: kc_user.last_name,
                    avatar_url,
                })
            }))
            .collect()
    } else {
        let mut conn = db.get_conn().await?;

        let found_users = User::find(
            &mut conn,
            current_tenant.id,
            &query.q,
            MAX_USER_SEARCH_RESULTS,
        )
        .await?;

        found_users
            .into_iter()
            .map(|user| GetFindResponseEntry::Registered(user.to_public_user_profile(&settings)))
            .collect()
    };

    Ok(Json(GetFindResponseBody(found_users)))
}
