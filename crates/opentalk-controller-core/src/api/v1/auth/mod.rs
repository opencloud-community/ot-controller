// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Auth related API structs and Endpoints
use core::mem::take;

use actix_web::{
    get, post,
    web::{Data, Json},
};
use kustos::prelude::PoliciesBuilder;
use log::error;
use opentalk_controller_settings::{TariffAssignment, TariffStatusMapping, TenantAssignment};
use opentalk_database::{Db, OptionalExt};
use opentalk_db_storage::{
    groups::{get_or_create_groups_by_name, Group},
    tariffs::{ExternalTariffId, Tariff},
    tenants::{get_or_create_tenant_by_oidc_id, OidcTenantId},
    users::User,
};
use opentalk_types::api::error::{ApiError, AuthenticationError};
use opentalk_types_api_v1::auth::{
    login::AuthLoginPostRequestBody, GetLoginResponseBody, OidcProvider, PostLoginResponseBody,
};
use opentalk_types_common::{
    events::EventId,
    rooms::RoomId,
    tariffs::TariffStatus,
    tenants::TenantId,
    users::{DisplayName, GroupName},
};

use super::{events::EventPoliciesBuilderExt, rooms::RoomsPoliciesBuilderExt};
use crate::{
    api::responses::InternalServerError,
    oidc::{IdTokenInfo, OidcContext, VerifyError},
    settings::SharedSettingsActix,
};

mod create_user;
mod update_user;

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
pub async fn post_login(
    settings: SharedSettingsActix,
    db: Data<Db>,
    oidc_ctx: Data<OidcContext>,
    body: Json<AuthLoginPostRequestBody>,
    authz: Data<kustos::Authz>,
) -> Result<Json<PostLoginResponseBody>, ApiError> {
    let id_token = body.into_inner().id_token;

    let mut info = match oidc_ctx.verify_id_token(&id_token) {
        Ok(info) => info,
        Err(e) => {
            return match e {
                VerifyError::InvalidClaims => Err(ApiError::bad_request()
                    .with_code("invalid_claims")
                    .with_message("some required attributes are missing or malformed")),
                VerifyError::Expired { .. } => Err(ApiError::unauthorized()
                    .with_www_authenticate(AuthenticationError::SessionExpired)),
                VerifyError::MissingKeyID
                | VerifyError::UnknownKeyID
                | VerifyError::MalformedSignature
                | VerifyError::InvalidJwt { .. }
                | VerifyError::InvalidSignature => Err(ApiError::unauthorized()
                    .with_www_authenticate(AuthenticationError::InvalidIdToken)),
            };
        }
    };

    let settings = settings.load_full();
    let mut conn = db.get_conn().await?;

    // Get tariff depending on the configured assignment
    let (tariff, tariff_status) = match &settings.tariffs.assignment {
        TariffAssignment::Static { static_tariff_name } => (
            Tariff::get_by_name(&mut conn, static_tariff_name).await?,
            TariffStatus::Default,
        ),
        TariffAssignment::ByExternalTariffId => {
            let external_tariff_id = info.tariff_id.clone().ok_or_else(|| {
                ApiError::bad_request()
                    .with_code("invalid_claims")
                    .with_message("tariff_id missing in id_token claims")
            })?;

            let tariff =
                Tariff::get_by_external_id(&mut conn, &ExternalTariffId::from(external_tariff_id))
                    .await
                    .optional()?
                    .ok_or_else(|| {
                        ApiError::internal()
                            .with_code("invalid_tariff_id")
                            .with_message("JWT contained unknown tariff_id")
                    })?;

            if let Some(mapping) = settings.tariffs.status_mapping.as_ref() {
                let status_name = info.tariff_status.clone().ok_or_else(|| {
                    ApiError::bad_request()
                        .with_code("invalid_claims")
                        .with_message("tariff_status missing in id_token claims")
                })?;

                let status = map_tariff_status_name(mapping, &status_name);

                let tariff = if matches!(status, TariffStatus::Downgraded) {
                    Tariff::get_by_name(&mut conn, &mapping.downgraded_tariff_name)
                        .await
                        .map_err(|_| {
                            ApiError::internal()
                                .with_code("invalid_configuration")
                                .with_message("Unable to load downgraded tariff")
                        })?
                } else {
                    tariff
                };
                (tariff, status)
            } else {
                (tariff, TariffStatus::Default)
            }
        }
    };

    // Get the tenant_id depending on the configured assignment
    let tenant_id = match &settings.tenants.assignment {
        TenantAssignment::Static { static_tenant_id } => static_tenant_id.clone(),
        TenantAssignment::ByExternalTenantId { .. } => info.tenant_id.clone().ok_or_else(|| {
            ApiError::bad_request()
                .with_code("invalid_claims")
                .with_message("tenant_id missing in id_token claims")
        })?,
    };

    let tenant = get_or_create_tenant_by_oidc_id(&mut conn, &OidcTenantId::from(tenant_id)).await?;

    let groups: Vec<(TenantId, GroupName)> = take(&mut info.x_grp)
        .into_iter()
        .map(|group| (tenant.id, GroupName::from(group)))
        .collect();

    let groups = get_or_create_groups_by_name(&mut conn, &groups).await?;

    // Try to get the user by the `sub` field in the IdToken
    let user = User::get_by_oidc_sub(&mut conn, tenant.id, &info.sub).await?;

    let login_result = match user {
        Some(user) => {
            // Found a matching user, update its attributes, tenancy and groups
            update_user::update_user(
                &settings,
                &mut conn,
                user,
                info,
                groups,
                tariff,
                tariff_status,
            )
            .await?
        }
        None => {
            // No matching user, create a new one with inside the given tenants and groups
            create_user::create_user(
                &settings,
                &mut conn,
                info,
                tenant,
                groups,
                tariff,
                tariff_status,
            )
            .await?
        }
    };

    drop(conn);

    update_core_user_permissions(authz.as_ref(), login_result).await?;

    Ok(Json(PostLoginResponseBody {
        // TODO calculate permissions
        permissions: Default::default(),
    }))
}

fn map_tariff_status_name(mapping: &TariffStatusMapping, name: &String) -> TariffStatus {
    if mapping.default.contains(name) {
        TariffStatus::Default
    } else if mapping.paid.contains(name) {
        TariffStatus::Paid
    } else if mapping.downgraded.contains(name) {
        TariffStatus::Downgraded
    } else {
        error!("Invalid tariff status value found: \"{name}\"");
        TariffStatus::Default
    }
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
pub async fn get_login(oidc_ctx: Data<OidcContext>) -> Json<GetLoginResponseBody> {
    let provider = OidcProvider {
        name: "default".to_string(),
        url: oidc_ctx.provider_url(),
    };

    Json(GetLoginResponseBody { oidc: provider })
}

enum LoginResult {
    UserCreated {
        user: User,
        groups: Vec<Group>,
        event_and_room_ids: Vec<(EventId, RoomId)>,
    },
    UserUpdated {
        user: User,
        groups_added_to: Vec<Group>,
        groups_removed_from: Vec<Group>,
    },
}

async fn update_core_user_permissions(
    authz: &kustos::Authz,
    db_result: LoginResult,
) -> Result<(), ApiError> {
    match db_result {
        LoginResult::UserUpdated {
            user,
            groups_added_to,
            groups_removed_from,
        } => {
            // TODO(r.floren) this could be optimized I guess, with a user_to_groups?
            // But this is currently not a hot path.
            for group in groups_added_to {
                authz.add_user_to_group(user.id, group.id).await?;
            }

            for group in groups_removed_from {
                authz.remove_user_from_group(user.id, group.id).await?;
            }
        }
        LoginResult::UserCreated {
            user,
            groups,
            event_and_room_ids,
        } => {
            authz.add_user_to_role(user.id, "user").await?;

            for group in groups {
                authz.add_user_to_group(user.id, group.id).await?;
            }

            // Migrate email invites to user invites
            // Add permissions for user to events that the email was invited to
            if event_and_room_ids.is_empty() {
                return Ok(());
            }

            let mut policies = PoliciesBuilder::new().grant_user_access(user.id);

            for (event_id, room_id) in event_and_room_ids {
                policies = policies
                    .event_read_access(event_id)
                    .room_read_access(room_id)
                    .event_invite_invitee_access(event_id);
            }

            authz.add_policies(policies.finish()).await?;
        }
    }

    Ok(())
}

fn build_info_display_name(info: &IdTokenInfo) -> DisplayName {
    DisplayName::from_str_lossy(
        &info
            .display_name
            .clone()
            .unwrap_or_else(|| format!("{} {}", &info.firstname, &info.lastname)),
    )
}
