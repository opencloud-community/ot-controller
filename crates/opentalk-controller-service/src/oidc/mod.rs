// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Provides OpenID Connect stuff.

use chrono::{DateTime, Utc};
use http::async_http_client;
use openidconnect::{AccessToken, ClientId, ClientSecret, TokenIntrospectionResponse};
use provider::ProviderClient;
use snafu::ResultExt;
use url::Url;

use crate::Result;

mod claims;
mod http;
mod jwt;
mod provider;

pub use claims::{ServiceClaims, UserClaims};
pub use jwt::{decode_token, VerifyError};

/// The `OidcContext` contains all information about the Oidc provider and permissions matrix.
#[derive(Debug)]
pub struct OidcContext {
    /// The URL used by the frontend for authentication
    pub frontend_auth_base_url: Url,
    /// The provider client
    pub provider: ProviderClient,
    /// The HTTP client
    http_client: reqwest11::Client,
}

impl OidcContext {
    /// Creates the OidcContext.
    /// This reads the OIDC provider configuration and tries to fetch the metadata from it.
    /// If a provider is misconfigured or not reachable this function will fail.
    #[tracing::instrument(name = "oidc_discover", skip(client_secret))]
    pub async fn new(
        frontend_auth_base_url: Url,
        controller_auth_base_url: Url,
        client_id: ClientId,
        client_secret: ClientSecret,
    ) -> Result<Self> {
        let http_client = http::make_client().whatever_context("Failed to make http client")?;

        let client = ProviderClient::discover(
            http_client.clone(),
            controller_auth_base_url,
            client_id,
            client_secret,
        )
        .await
        .whatever_context("Failed to discover provider client")?;

        Ok(Self {
            frontend_auth_base_url,
            provider: client,
            http_client,
        })
    }

    /// Verifies the signature and expiration of an AccessToken.
    ///
    /// Returns the subject (user id) if the token is verified.
    ///
    /// Note: This does __not__ check if the token is active or has been revoked.
    /// See `verify_access_token_active`.
    #[tracing::instrument(name = "oidc_verify_access_token", skip(self, access_token))]
    pub fn verify_access_token<C: jwt::VerifyClaims>(
        &self,
        access_token: &AccessToken,
    ) -> Result<C, VerifyError> {
        jwt::verify::<C>(
            self.provider.metadata.jwks(),
            access_token.secret().as_str(),
        )
    }

    /// Verify that an AccessToken is active using the providers `token_introspect` endpoint.
    ///
    /// Returns an error if it fails to validate the token.
    ///
    /// If the function returns Ok(_) the caller must inspect the returned [AccessTokenIntrospectInfo]
    /// to check if the AccessToken is still active.
    #[tracing::instrument(name = "oidc_introspect_access_token", skip(self, access_token), fields(active = tracing::field::Empty))]
    pub async fn introspect_access_token(
        &self,
        access_token: &AccessToken,
    ) -> Result<AccessTokenIntrospectInfo> {
        let response = self
            .provider
            .client
            .introspect(access_token)
            .whatever_context("Invalid access token")?
            .request_async(async_http_client(self.http_client.clone()))
            .await
            .whatever_context("Failed to verify token using the introspect endpoint")?;

        _ = tracing::Span::current().record("active", response.active());

        Ok(AccessTokenIntrospectInfo {
            active: response.active(),
        })
    }

    /// Verifies the signature and expiration of the ID Token and returns related info
    ///
    /// Returns an error if `id_token` is invalid or expired
    #[tracing::instrument(name = "oidc_verify_id_token", skip(self, id_token))]
    pub fn verify_id_token(&self, id_token: &str) -> Result<IdTokenInfo, VerifyError> {
        let claims = jwt::verify::<UserClaims>(self.provider.metadata.jwks(), id_token)?;

        Ok(IdTokenInfo {
            sub: claims.sub,
            issuer: claims.iss,
            expiration: claims.exp,
            email: claims.email.to_lowercase().into(),
            firstname: claims.given_name,
            lastname: claims.family_name,
            avatar_url: claims.picture,
            x_grp: claims.x_grp,
            phone_number: claims.phone_number,
            display_name: claims.nickname,
            tenant_id: claims.tenant_id,
            tariff_id: claims.tariff_id,
            tariff_status: claims.tariff_status,
        })
    }

    /// Returns the provider URL
    pub fn provider_url(&self) -> String {
        self.frontend_auth_base_url.to_string()
    }
}

/// Relevant info returned from `verify_access_token_active` function.
#[derive(Debug)]
#[must_use]
pub struct AccessTokenIntrospectInfo {
    /// Indicates whether it's active
    pub active: bool,
}

/// The result of an successful ID Token verification.
///
/// Contains the sub (client id) and expiration of the ID Token
#[derive(Debug)]
pub struct IdTokenInfo {
    /// The subject
    pub sub: String,
    /// The subject
    pub issuer: String,
    /// The date and time of expiration
    pub expiration: DateTime<Utc>,
    /// The email address
    pub email: String,
    /// The first name
    pub firstname: String,
    /// The last name
    pub lastname: String,
    /// The URL to get the avatar from
    pub avatar_url: Option<String>,
    /// The group
    pub x_grp: Vec<String>,
    /// The phone number
    pub phone_number: Option<String>,
    /// The display name
    pub display_name: Option<String>,
    /// The tenant id
    pub tenant_id: Option<String>,
    /// The tariff id
    pub tariff_id: Option<String>,
    /// The tariff status
    pub tariff_status: Option<String>,
}
