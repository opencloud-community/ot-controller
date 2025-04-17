// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use openidconnect::{
    core::CoreClient, url::Url, ClientId, ClientSecret, IntrospectionUrl, IssuerUrl,
};
use serde::{Deserialize, Serialize};
use snafu::{whatever, ResultExt};

use super::http::async_http_client;
use crate::Result;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdditionalProviderMetadata {
    pub introspection_endpoint: Option<Url>,
}

impl openidconnect::AdditionalProviderMetadata for AdditionalProviderMetadata {}

type ProviderMetadata = openidconnect::ProviderMetadata<
    AdditionalProviderMetadata,
    openidconnect::core::CoreAuthDisplay,
    openidconnect::core::CoreClientAuthMethod,
    openidconnect::core::CoreClaimName,
    openidconnect::core::CoreClaimType,
    openidconnect::core::CoreGrantType,
    openidconnect::core::CoreJweContentEncryptionAlgorithm,
    openidconnect::core::CoreJweKeyManagementAlgorithm,
    openidconnect::core::CoreJwsSigningAlgorithm,
    openidconnect::core::CoreJsonWebKeyType,
    openidconnect::core::CoreJsonWebKeyUse,
    openidconnect::core::CoreJsonWebKey,
    openidconnect::core::CoreResponseMode,
    openidconnect::core::CoreResponseType,
    openidconnect::core::CoreSubjectIdentifierType,
>;

/// Contains all structures necessary to talk to a single configured OIDC Provider.
#[derive(Debug)]
pub struct ProviderClient {
    pub metadata: ProviderMetadata,
    pub client: CoreClient,
}

impl ProviderClient {
    /// Discover Provider information from given settings
    pub async fn discover(
        http_client: reqwest11::Client,
        auth_base_url: Url,
        client_id: ClientId,
        client_secret: ClientSecret,
    ) -> Result<ProviderClient> {
        let metadata = ProviderMetadata::discover_async(
            IssuerUrl::from_url(auth_base_url),
            async_http_client(http_client),
        )
        .await
        .whatever_context("Failed to discover provider metadata")?;

        // Require the userinfo endpoint
        if metadata.userinfo_endpoint().is_none() {
            whatever!("OpenID Connect provider is missing the 'userinfo' endpoint");
        }

        let mut client = CoreClient::new(
            client_id.clone(),
            Some(client_secret),
            metadata.issuer().clone(),
            metadata.authorization_endpoint().clone(),
            metadata.token_endpoint().cloned(),
            metadata.userinfo_endpoint().cloned(),
            metadata.jwks().clone(),
        );

        // Optionally support the introspection endpoint
        if let Some(introspection_url) = &metadata.additional_metadata().introspection_endpoint {
            client =
                client.set_introspection_uri(IntrospectionUrl::from_url(introspection_url.clone()));
        }

        Ok(ProviderClient { metadata, client })
    }
}
