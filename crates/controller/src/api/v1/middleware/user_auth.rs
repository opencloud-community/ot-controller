// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Handles user Authentication in API requests
use crate::api::v1::middleware::user_auth::bearer_or_invite_code::BearerOrInviteCode;
use crate::api::v1::response::error::{AuthenticationError, CacheableApiError};
use crate::api::v1::response::ApiError;
use crate::caches::Caches;
use crate::oidc::{OidcContext, UserClaims};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::error::Error;
use actix_web::http::header::Header;
use actix_web::web::Data;
use actix_web::{HttpMessage, ResponseError};
use actix_web_httpauth::headers::authorization::Authorization;
use cache::Cache;
use chrono::Utc;
use controller_settings::{Settings, SharedSettings, TenantAssignment};
use core::future::ready;
use database::Db;
use db_storage::tenants::{OidcTenantId, Tenant};
use db_storage::users::User;
use openidconnect::AccessToken;
use std::future::{Future, Ready};
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use tracing_futures::Instrument;
use types::core::InviteCodeId;
use uuid::Uuid;

mod bearer_or_invite_code;

pub type UserAccessTokenCache = Cache<String, Result<(Tenant, User), CacheableApiError>>;

/// Middleware factory
///
/// Transforms into [`OidcAuthMiddleware`]
pub struct OidcAuth {
    pub settings: SharedSettings,
    pub db: Data<Db>,
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
            settings: self.settings.clone(),
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
    settings: SharedSettings,
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
        let settings = self.settings.clone();
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
                log::warn!("Unable to parse access token or invite code, {}", e);
                let error = ApiError::unauthorized()
                    .with_message("Unable to parse access token or invite code")
                    .with_www_authenticate(AuthenticationError::InvalidAccessToken);

                let response = req.into_response(error.error_response());
                return Box::pin(ready(Ok(response)));
            }
        };

        enum AccessTokenOrInviteCode {
            AccessToken(AccessToken),
            InviteCode(InviteCodeId),
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
                let settings = settings.load_full();

                match access_token_or_invite_code {
                    AccessTokenOrInviteCode::AccessToken(access_token) => match check_access_token(
                        &settings,
                        db,
                        oidc_ctx,
                        &caches.user_access_tokens,
                        access_token,
                    )
                    .await
                    {
                        Ok((current_tenant, current_user)) => {
                            req.extensions_mut()
                                .insert(kustos::actix_web::User::from(Uuid::from(current_user.id)));
                            req.extensions_mut().insert(current_tenant);
                            req.extensions_mut().insert(current_user);
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

#[tracing::instrument(skip_all)]
pub async fn check_access_token(
    settings: &Settings,
    db: Data<Db>,
    oidc_ctx: Data<OidcContext>,
    cache: &UserAccessTokenCache,
    access_token: AccessToken,
) -> Result<(Tenant, User), ApiError> {
    // First verify the access-token's signature, expiry and claims
    let user_claims = match oidc_ctx.verify_access_token::<UserClaims>(&access_token) {
        Ok(user_claims) => user_claims,
        Err(e) => {
            log::debug!("Invalid access token, {}", e);
            return Err(ApiError::unauthorized()
                .with_www_authenticate(AuthenticationError::InvalidAccessToken));
        }
    };

    // Access token is still valid, search for cached results
    if let Ok(Some(result)) = cache.get(access_token.secret()).await {
        // Hit! Return the cached result
        match result {
            Ok(tenant_and_user) => Ok(tenant_and_user),
            Err(err) => Err(ApiError::from_cacheable(err)?),
        }
    } else {
        // Miss, do the check and cache the result

        // Calculate the remaining ttl of the token
        let token_ttl = user_claims.exp - Utc::now();

        let check_result =
            check_access_token_inner(settings, db, oidc_ctx, &access_token, user_claims).await;

        match check_result {
            Ok((tenant, user)) => {
                // Check if the id-token info is expired
                //
                // This result must not be cached!
                // The error can be resolved by calling /auth/login with a valid id_token.
                // The same access-token will then pass this check here.
                if chrono::Utc::now().timestamp() > user.id_token_exp {
                    return Err(ApiError::unauthorized()
                        .with_message("The session for this user has expired")
                        .with_www_authenticate(AuthenticationError::SessionExpired));
                }

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
                            Err(e.to_cacheable()),
                            token_ttl.to_std().expect("duration was previously checked"),
                        )
                        .await?;
                }

                Err(e)
            }
        }
    }
}

/// Fetches all associated user data of the access token
async fn check_access_token_inner(
    settings: &Settings,
    db: Data<Db>,
    oidc_ctx: Data<OidcContext>,
    access_token: &AccessToken,
    claims: UserClaims,
) -> Result<(Tenant, User), ApiError> {
    // Get the tenant_id depending on the configured assignment
    let oidc_tenant_id = match &settings.tenants.assignment {
        TenantAssignment::Static { static_tenant_id } => static_tenant_id.clone(),
        TenantAssignment::ByExternalTenantId { .. } => claims.tenant_id.ok_or_else(|| {
            log::error!("Invalid access token, missing tenant_id");
            ApiError::unauthorized().with_www_authenticate(AuthenticationError::InvalidAccessToken)
        })?,
    };

    let mut conn = db.get_conn().await?;

    let current_tenant = Tenant::get_by_oidc_id(&mut conn, OidcTenantId::from(oidc_tenant_id))
        .await?
        .ok_or_else(|| {
            ApiError::unauthorized()
                .with_code("unknown_tenant_id")
                .with_message("Unknown tenant_id in access token. Please login first!")
        })?;

    let current_user = User::get_by_oidc_sub(&mut conn, current_tenant.id, &claims.sub)
        .await?
        .ok_or_else(|| {
            ApiError::unauthorized()
                .with_code("unknown_sub")
                .with_message("Unknown subject in access token. Please login first!")
        })?;

    drop(conn);

    let info = match oidc_ctx.introspect_access_token(access_token).await {
        Ok(info) => info,
        Err(e) => {
            log::error!("Failed to check if AccessToken is active, {}", e);
            return Err(ApiError::internal());
        }
    };

    if info.active {
        Ok((current_tenant, current_user))
    } else {
        Err(ApiError::unauthorized()
            .with_www_authenticate(AuthenticationError::AccessTokenInactive))
    }
}
