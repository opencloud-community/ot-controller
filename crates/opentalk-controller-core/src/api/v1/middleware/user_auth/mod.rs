// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Handles user Authentication in API requests
use core::future::ready;
use std::{
    future::{Future, Ready},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::Error,
    http::header::Header,
    web::Data,
    HttpMessage, ResponseError,
};
use actix_web_httpauth::headers::authorization::Authorization;
use chrono::{DateTime, Utc};
use kustos::prelude::PoliciesBuilder;
use openidconnect::AccessToken;
use opentalk_cache::Cache;
use opentalk_controller_service::{
    controller_backend::RoomsPoliciesBuilderExt,
    oidc::{OidcContext, OnlyExpiryClaim, OpenIdConnectUserInfo},
};
use opentalk_controller_service_facade::RequestUser;
use opentalk_controller_settings::{
    Settings, SettingsProvider, TariffAssignment, TariffStatusMapping, TenantAssignment,
};
use opentalk_controller_utils::CaptureApiError;
use opentalk_database::{Db, OptionalExt};
use opentalk_db_storage::{
    groups::{get_or_create_groups_by_name, Group},
    tariffs::{ExternalTariffId, Tariff},
    tenants::{get_or_create_tenant_by_oidc_id, OidcTenantId, Tenant},
    users::User,
};
use opentalk_types_api_v1::error::{ApiError, AuthenticationError};
use opentalk_types_common::{
    events::EventId,
    rooms::{invite_codes::InviteCode, RoomId},
    tariffs::TariffStatus,
    tenants::TenantId,
    users::{DisplayName, GroupName},
};
use snafu::Report;
use tracing_futures::Instrument;
use uuid::Uuid;

use crate::{
    api::v1::{
        events::EventPoliciesBuilderExt,
        middleware::user_auth::bearer_or_invite_code::BearerOrInviteCode,
        response::error::CacheableApiError,
    },
    caches::Caches,
};

mod bearer_or_invite_code;
mod create_user;
mod update_user;

pub type UserAccessTokenCache = Cache<String, Result<(Tenant, User), CacheableApiError>>;

/// Middleware factory
///
/// Transforms into [`OidcAuthMiddleware`]
pub struct OidcAuth {
    pub settings_provider: SettingsProvider,
    pub db: Data<Db>,
    pub authz: Data<kustos::Authz>,
    pub oidc_ctx: Data<OidcContext>,
}

impl<S> Transform<S, ServiceRequest> for OidcAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = OidcAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(OidcAuthMiddleware {
            service: Rc::new(service),
            settings_provider: self.settings_provider.clone(),
            authz: self.authz.clone(),
            db: self.db.clone(),
            oidc_ctx: self.oidc_ctx.clone(),
        }))
    }
}

/// Authentication middleware
///
/// Whenever an API request is received, the OidcAuthMiddleware will validate the access
/// token and provide the associated user as [`ReqData`](actix_web::web::ReqData) for the subsequent services.
pub struct OidcAuthMiddleware<S> {
    service: Rc<S>,
    settings_provider: SettingsProvider,
    authz: Data<kustos::Authz>,
    db: Data<Db>,
    oidc_ctx: Data<OidcContext>,
}

type ResultFuture<O, E> = Pin<Box<dyn Future<Output = Result<O, E>>>>;

impl<S> Service<ServiceRequest> for OidcAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = ResultFuture<Self::Response, Self::Error>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let settings_provider = self.settings_provider.clone();
        let authz = self.authz.clone();
        let db = self.db.clone();
        let oidc_ctx = self.oidc_ctx.clone();
        let caches = req
            .app_data::<Data<Caches>>()
            .expect("Caches must be provided as AppData")
            .clone();

        let parse_match_span = tracing::span!(
            tracing::Level::TRACE,
            "Authorization::<BearerOrInviteCode>::parse"
        );

        let _enter = parse_match_span.enter();
        let auth = match Authorization::<BearerOrInviteCode>::parse(&req) {
            Ok(a) => a,
            Err(e) => {
                log::warn!(
                    "Unable to parse access token or invite code, {}",
                    Report::from_error(e)
                );
                let error = ApiError::unauthorized()
                    .with_message("Unable to parse access token or invite code")
                    .with_www_authenticate(AuthenticationError::InvalidAccessToken);

                let response = req.into_response(error.error_response());
                return Box::pin(ready(Ok(response)));
            }
        };

        enum AccessTokenOrInviteCode {
            AccessToken(AccessToken),
            InviteCode(InviteCode),
        }

        let access_token_or_invite_code = match auth.into_scheme() {
            BearerOrInviteCode::Bearer(bearer) => {
                AccessTokenOrInviteCode::AccessToken(AccessToken::new(bearer.token().to_string()))
            }
            BearerOrInviteCode::InviteCode(invite_code) => {
                AccessTokenOrInviteCode::InviteCode(invite_code)
            }
        };

        Box::pin(
            async move {
                let settings = settings_provider.get();

                match access_token_or_invite_code {
                    AccessTokenOrInviteCode::AccessToken(access_token) => match check_access_token(
                        &settings,
                        &authz,
                        db,
                        oidc_ctx,
                        &caches.user_access_tokens,
                        &access_token,
                    )
                    .await
                    {
                        Ok((current_tenant, current_user)) => {
                            req.extensions_mut()
                                .insert(kustos::actix_web::User::from(Uuid::from(current_user.id)));
                            req.extensions_mut().insert(current_tenant);
                            req.extensions_mut().insert(current_user.clone());
                            req.extensions_mut()
                                .insert(build_request_user(current_user));
                            req.extensions_mut().insert(access_token);
                            service.call(req).await
                        }
                        Err(err) => Ok(req.into_response(err.error_response())),
                    },
                    AccessTokenOrInviteCode::InviteCode(current_invite_code) => {
                        req.extensions_mut()
                            .insert(kustos::actix_web::Invite::from(Uuid::from(
                                current_invite_code,
                            )));
                        req.extensions_mut().insert(current_invite_code);
                        service.call(req).await
                    }
                }
            }
            .instrument(tracing::trace_span!("OidcAuthMiddleware::async::call")),
        )
    }
}

fn build_request_user(user: User) -> RequestUser {
    RequestUser {
        id: user.id,
        email: user.email,
        title: user.title,
        firstname: user.firstname,
        lastname: user.lastname,
        language: user.language,
        display_name: user.display_name,
        dashboard_theme: user.dashboard_theme,
        conference_theme: user.conference_theme,
        tenant_id: user.tenant_id,
        tariff_id: user.tariff_id,
        tariff_status: user.tariff_status,
        avatar_url: user.avatar_url,
    }
}

#[tracing::instrument(skip_all)]
pub async fn check_access_token(
    settings: &Settings,
    authz: &kustos::Authz,
    db: Data<Db>,
    oidc_ctx: Data<OidcContext>,
    cache: &UserAccessTokenCache,
    access_token: &AccessToken,
) -> Result<(Tenant, User), CaptureApiError> {
    // Search for cached results
    if let Ok(Some(result)) = cache.get(access_token.secret()).await {
        // Hit! Return the cached result
        match result {
            Ok(tenant_and_user) => Ok(tenant_and_user),
            Err(err) => Err(CaptureApiError::try_from(err).map_err(|e| {
                log::warn!("Error while creating error: {}", Report::from_error(e));
                CaptureApiError::from(ApiError::internal())
            })?),
        }
    } else {
        // Miss, do the check and cache the result

        let expires_at = verify_access_token(&oidc_ctx, access_token).await?;

        // Calculate the remaining ttl of the token
        let token_ttl = expires_at - Utc::now();

        let check_result =
            check_access_token_inner(settings, authz, db, oidc_ctx, access_token).await;

        match check_result {
            Ok((tenant, user)) => {
                // Avoid caching results for tokens that are about to expire
                if token_ttl > chrono::Duration::seconds(10) {
                    cache
                        .insert_with_ttl(
                            access_token.secret().clone(),
                            Ok((tenant.clone(), user.clone())),
                            token_ttl.to_std().expect("duration was previously checked"),
                        )
                        .await?;
                }

                Ok((tenant, user))
            }
            Err(e) => {
                // Do not cache internal errors or tokens that are about to expire
                if !e.status_code().is_server_error() && token_ttl > chrono::Duration::seconds(10) {
                    cache
                        .insert_with_ttl(
                            access_token.secret().clone(),
                            Err(CacheableApiError::from(&e)),
                            token_ttl.to_std().expect("duration was previously checked"),
                        )
                        .await?;
                }

                Err(e)
            }
        }
    }
}

/// Verify the access token and return it's expiry timestamp.
///
/// Uses the introspect endpoint if available (https://www.rfc-editor.org/rfc/rfc7662),
/// otherwise the token must be a JWT.
async fn verify_access_token(
    oidc_ctx: &OidcContext,
    access_token: &AccessToken,
) -> Result<DateTime<Utc>, CaptureApiError> {
    if oidc_ctx.supports_introspect() {
        let introspect_info = match oidc_ctx.introspect(access_token.clone()).await {
            Ok(introspect_info) => introspect_info,
            Err(e) => {
                log::error!(
                    "Failed to check if AccessToken is active, {}",
                    Report::from_error(e)
                );

                return Err(ApiError::internal().into());
            }
        };

        if !introspect_info.active {
            return Err(ApiError::unauthorized()
                .with_www_authenticate(AuthenticationError::AccessTokenInactive)
                .into());
        }

        return Ok(introspect_info.exp);
    }

    // If there's no introspect endpoint, the token must be a JWT with an exp field
    match oidc_ctx.verify_jwt_token::<OnlyExpiryClaim>(access_token) {
        Ok(jwt_claims) => Ok(jwt_claims.exp),
        Err(e) => {
            log::debug!("Invalid access token, {}", Report::from_error(e));
            Err(ApiError::unauthorized()
                .with_www_authenticate(AuthenticationError::InvalidAccessToken)
                .into())
        }
    }
}

/// Fetches all associated user data of the access token
async fn check_access_token_inner(
    settings: &Settings,
    authz: &kustos::Authz,
    db: Data<Db>,
    oidc_ctx: Data<OidcContext>,
    access_token: &AccessToken,
) -> Result<(Tenant, User), CaptureApiError> {
    let info = oidc_ctx.user_info(access_token.clone()).await?;

    let mut conn = db.get_conn().await?;

    // Get tariff depending on the configured assignment
    let (tariff, tariff_status) = match &settings.tariffs.assignment {
        TariffAssignment::Static { static_tariff_name } => (
            Tariff::get_by_name(&mut conn, static_tariff_name).await?,
            TariffStatus::Default,
        ),
        TariffAssignment::ByExternalTariffId { status_mapping } => {
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

            if let Some(mapping) = status_mapping.as_ref() {
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
            log::error!("Invalid access token, missing tenant_id");
            ApiError::unauthorized().with_www_authenticate(AuthenticationError::InvalidAccessToken)
        })?,
    };
    let tenant = get_or_create_tenant_by_oidc_id(&mut conn, &OidcTenantId::from(tenant_id)).await?;

    let groups: Vec<(TenantId, GroupName)> = info
        .groups
        .iter()
        .map(|group| (tenant.id, GroupName::from(group.clone())))
        .collect();
    let groups = get_or_create_groups_by_name(&mut conn, &groups).await?;

    let user = User::get_by_oidc_sub(&mut conn, tenant.id, &info.sub).await?;

    let login_result = match user {
        Some(user) => {
            // Found a matching user, update its attributes, tenancy and groups
            update_user::update_user(
                settings,
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
                settings,
                &mut conn,
                info,
                &tenant,
                groups,
                tariff,
                tariff_status,
            )
            .await?
        }
    };

    let user = update_core_user_permissions(authz, login_result).await?;

    Ok((tenant, user))
}

fn map_tariff_status_name(mapping: &TariffStatusMapping, name: &String) -> TariffStatus {
    if mapping.default.contains(name) {
        TariffStatus::Default
    } else if mapping.paid.contains(name) {
        TariffStatus::Paid
    } else if mapping.downgraded.contains(name) {
        TariffStatus::Downgraded
    } else {
        log::error!("Invalid tariff status value found: \"{name}\"");
        TariffStatus::Default
    }
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
) -> Result<User, CaptureApiError> {
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

            Ok(user)
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
                return Ok(user);
            }

            let mut policies = PoliciesBuilder::new().grant_user_access(user.id);

            for (event_id, room_id) in event_and_room_ids {
                policies = policies
                    .event_read_access(event_id)
                    .room_read_access(room_id)
                    .event_invite_invitee_access(event_id);
            }

            authz.add_policies(policies.finish()).await?;

            Ok(user)
        }
    }
}

fn build_info_display_name(info: &OpenIdConnectUserInfo) -> DisplayName {
    DisplayName::from_str_lossy(
        &info
            .display_name
            .clone()
            .unwrap_or_else(|| format!("{} {}", &info.firstname, &info.lastname)),
    )
}
