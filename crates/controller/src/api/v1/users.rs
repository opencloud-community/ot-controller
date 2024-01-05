// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! User related API structs and Endpoints
//!
//! The defined structs are exposed to the REST API and will be serialized/deserialized. Similar
//! structs are defined in the Database crate [`db_storage`] for database operations.

use super::response::{ApiError, NoContent};
use crate::api::signaling::SignalingModules;
use crate::settings::SharedSettingsActix;
use actix_web::web::{Data, Json, Path, Query, ReqData};
use actix_web::{get, patch, Either};
use anyhow::Context;
use controller_settings::TenantAssignment;
use database::Db;
use db_storage::tariffs::Tariff;
use db_storage::tenants::Tenant;
use db_storage::users::{email_to_libravatar_url, UpdateUser, User};
use keycloak_admin::KeycloakAdminClient;
use types::{
    api::v1::users::{
        GetFindQuery, GetFindResponseItem, PatchMeBody, PrivateUserProfile, PublicUserProfile,
        UnregisteredUser,
    },
    common::tariff::TariffResource,
    core::UserId,
};
use validator::Validate;

const MAX_USER_SEARCH_RESULTS: usize = 20;

/// API Endpoint *PATCH /users/me*
#[patch("/users/me")]
pub async fn patch_me(
    settings: SharedSettingsActix,
    db: Data<Db>,
    current_user: ReqData<User>,
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
        title: patch.title.as_deref(),
        firstname: None,
        lastname: None,
        phone: None,
        email: None,
        display_name: patch.display_name.as_deref(),
        language: patch.language.as_deref(),
        dashboard_theme: patch.dashboard_theme.as_deref(),
        conference_theme: patch.conference_theme.as_deref(),
        id_token_exp: None,
        tariff_id: None,
        tariff_status: None,
    };

    let user = changeset.apply(&mut conn, current_user.id).await?;

    let user_profile = user.to_private_user_profile(&settings);

    Ok(Either::Left(Json(user_profile)))
}

/// API Endpoint *GET /users/me*
///
/// Returns the [`PrivateUserProfile`] of the requesting user.
#[get("/users/me")]
pub async fn get_me(
    settings: SharedSettingsActix,
    current_user: ReqData<User>,
) -> Result<Json<PrivateUserProfile>, ApiError> {
    let settings = settings.load_full();
    let current_user = current_user.into_inner();

    let user_profile = current_user.to_private_user_profile(&settings);

    Ok(Json(user_profile))
}

/// API Endpoint *GET /users/me/tariff*
///
/// Returns the [`TariffResource`] of the requesting user.
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
        settings.defaults.disabled_features(),
        modules.get_module_features(),
    );

    Ok(Json(response))
}

/// API Endpoint *GET /users/{user_id}*
///
/// Returns [`PublicUserProfile`] of the specified user
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

/// API Endpoint *GET /users/find?name=$input*
///
/// Returns a list with a limited size of users matching the query
#[get("/users/find")]
pub async fn find(
    settings: SharedSettingsActix,
    kc_admin_client: Data<KeycloakAdminClient>,
    db: Data<Db>,
    current_tenant: ReqData<Tenant>,
    query: Query<GetFindQuery>,
) -> Result<Json<Vec<GetFindResponseItem>>, ApiError> {
    let settings = settings.load_full();

    if settings.endpoints.disable_users_find {
        return Err(ApiError::not_found());
    }

    if query.q.len() < 3 {
        return Err(ApiError::bad_request()
            .with_code("query_too_short")
            .with_message("query must be at least 3 characters long"));
    }

    // Get all users from Keycloak matching the search criteria
    let found_users = if settings.endpoints.users_find_use_kc {
        let mut found_kc_users = match &settings.tenants.assignment {
            TenantAssignment::Static { .. } => {
                // Do not filter by tenant_id if the assignment is static, since that's used
                // when Keycloak does not provide any tenant information we can filter over anyway
                kc_admin_client
                    .search_user(&query.q, MAX_USER_SEARCH_RESULTS)
                    .await
                    .context("Failed to search for user in keycloak")?
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
                    .context("Failed to search for user in keycloak")?
            }
        };

        let mut conn = db.get_conn().await?;

        // Get the name of the user attribute providing the Keycloak user ids. If set to None,
        // Keycloak's id field is used for user assignment instead.
        let user_attribute_name = settings.keycloak.external_id_user_attribute_name.as_deref();

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
            .map(|user| GetFindResponseItem::Registered(user.to_public_user_profile(&settings)))
            .chain(found_kc_users.into_iter().map(|kc_user| {
                let avatar_url =
                    email_to_libravatar_url(&settings.avatar.libravatar_url, &kc_user.email);

                GetFindResponseItem::Unregistered(UnregisteredUser {
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
            .map(|user| GetFindResponseItem::Registered(user.to_public_user_profile(&settings)))
            .collect()
    };

    Ok(Json(found_users))
}
