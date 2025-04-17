// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Provides OpenID Connect stuff.

use std::ops::Deref;

use chrono::{DateTime, Utc};
use claims::OpenTalkAdditionalClaims;
use http::async_http_client;
use openidconnect::{
    core::CoreGenderClaim, AccessToken, ClientId, ClientSecret, LocalizedClaim,
    TokenIntrospectionResponse, UserInfoClaims,
};
use opentalk_controller_utils::CaptureApiError;
use opentalk_types_api_v1::error::ApiError;
use opentalk_types_common::time::TimeZone;
use provider::ProviderClient;
use snafu::{OptionExt, ResultExt, Whatever};
use url::Url;

use crate::Result;

mod claims;
mod http;
mod jwt;
mod provider;

pub use claims::{OnlyExpiryClaim, ServiceClaims};
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

    /// Verifies the signature and expiration of an AccessToken encoded as JWT (Json Web Token)
    ///
    /// This is used if the OpenID Connect Provider does not support introspection endpoints.
    #[tracing::instrument(name = "oidc_verify_access_token", skip(self, access_token))]
    pub fn verify_jwt_token<C: jwt::VerifyClaims>(
        &self,
        access_token: &AccessToken,
    ) -> Result<C, VerifyError> {
        jwt::verify::<C>(
            self.provider.metadata.jwks(),
            access_token.secret().as_str(),
        )
    }

    /// Returns if the configured provider support introspection
    pub fn supports_introspect(&self) -> bool {
        self.provider
            .metadata
            .additional_metadata()
            .introspection_endpoint
            .is_some()
    }

    /// Call the OIDC's userinfo endpoint to fetch the user data associated with the access token
    #[tracing::instrument(name = "oidc_user_info", skip_all)]
    pub async fn introspect(&self, access_token: AccessToken) -> Result<IntrospectInfo> {
        let claims = self
            .provider
            .client
            .introspect(&access_token)
            .whatever_context("Failed to build AccessToken introspect request")?
            .request_async(async_http_client(self.http_client.clone()))
            .await
            .whatever_context("AccessToken introspect request failed")?;

        Ok(IntrospectInfo {
            active: claims.active(),
            exp: claims
                .exp()
                .whatever_context("Introspection response does not contain 'exp' field")?,
        })
    }

    /// Call the OIDC's userinfo endpoint to fetch the user data associated with the access token
    #[tracing::instrument(err, name = "oidc_user_info", skip_all)]
    pub async fn user_info(
        &self,
        access_token: AccessToken,
    ) -> Result<OpenIdConnectUserInfo, CaptureApiError> {
        let claims: UserInfoClaims<OpenTalkAdditionalClaims, CoreGenderClaim> = self
            .provider
            .client
            .user_info(access_token, None)
            .whatever_context::<_, Whatever>("Failed to build userinfo request")?
            .request_async(async_http_client(self.http_client.clone()))
            .await
            .whatever_context::<_, Whatever>("Failed to fetch userinfo")?;

        let timezone_parse_result = claims.zoneinfo().map(|zi| (zi, zi.parse()));
        let timezone: Option<TimeZone> = match timezone_parse_result {
            // Zoneinfo exists and has correct IANA format
            Some((_, Ok(tz))) => Some(tz),
            // Zoneinfo exists but has wrong format
            Some((tz, Err(_))) => {
                log::warn!(
                    "Invalid zoneinfo value in token for OIDC sub {}: \"{}\"",
                    claims.subject().as_str(),
                    tz.as_str(),
                );
                None
            }
            None => None,
        };

        fn expect_present_localized<'a, T>(
            t: Option<&'a LocalizedClaim<T>>,
            name: &str,
        ) -> Result<&'a T, CaptureApiError> {
            match t.and_then(|c| c.get(None)) {
                Some(t) => Ok(t),
                None => Err(ApiError::bad_request()
                    .with_message(format!(
                        "userinfo claims are missing mandatory '{name}' field"
                    ))
                    .into()),
            }
        }

        fn expect_present<'a, T>(t: Option<&'a T>, name: &str) -> Result<&'a T, CaptureApiError> {
            match t {
                Some(t) => Ok(t),
                None => Err(ApiError::bad_request()
                    .with_message(format!(
                        "userinfo claims are missing mandatory '{name}' field"
                    ))
                    .into()),
            }
        }

        fn optional<T: Deref<Target = String>>(t: Option<&LocalizedClaim<T>>) -> Option<String> {
            t.and_then(|c| c.get(None)).map(|c| c.to_string())
        }

        Ok(OpenIdConnectUserInfo {
            sub: claims.subject().to_string(),
            email: expect_present(claims.email(), "email")?.to_string(),
            firstname: expect_present_localized(claims.given_name(), "given_name")?.to_string(),
            lastname: expect_present_localized(claims.family_name(), "family_name")?.to_string(),
            avatar_url: optional(claims.picture()),
            timezone,
            groups: claims.additional_claims().x_grp.clone(),
            phone_number: claims
                .phone_number()
                .map(|phone_number| phone_number.to_string()),
            display_name: optional(claims.nickname()),
            tenant_id: claims.additional_claims().tenant_id.clone(),
            tariff_id: claims.additional_claims().tariff_id.clone(),
            tariff_status: claims.additional_claims().tariff_status.clone(),
        })
    }

    /// Verifies the signature and expiration of an ID Token encoded as JWT (Json Web Token)
    ///
    /// Only used by the deprecated login endpoint
    #[tracing::instrument(name = "oidc_verify_id_token", skip_all)]
    pub fn verify_id_token(&self, id_token: &str) -> Result<(), VerifyError> {
        let _ = jwt::verify::<OnlyExpiryClaim>(self.provider.metadata.jwks(), id_token)?;
        Ok(())
    }

    /// Returns the provider URL
    pub fn provider_url(&self) -> String {
        self.frontend_auth_base_url.to_string()
    }
}

/// Info returned from the access token introspection
#[derive(Debug)]
#[must_use]
pub struct IntrospectInfo {
    /// Access token is still active
    pub active: bool,
    /// Expire timestamp of the token
    pub exp: DateTime<Utc>,
}

/// Relevant info returned from `userinfo` endpoint.
#[derive(Debug)]
#[must_use]
pub struct OpenIdConnectUserInfo {
    /// The users subject identifier, assigned by the OIDC provider
    pub sub: String,
    /// The email address
    pub email: String,
    /// The users firstname
    pub firstname: String,
    /// The last name
    pub lastname: String,
    /// The URL to get the avatar from
    pub avatar_url: Option<String>,
    /// The timezone of the user
    pub timezone: Option<TimeZone>,
    /// The groups
    pub groups: Vec<String>,
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
