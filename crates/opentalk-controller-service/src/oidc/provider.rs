// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use openidconnect::{
    core::CoreClient, url::Url, ClientId, ClientSecret, IntrospectionUrl, IssuerUrl,
};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use super::http::async_http_client;
use crate::Result;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdditionalProviderMetadata {
    introspection_endpoint: Url,
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

        let client = CoreClient::new(
            client_id.clone(),
            Some(client_secret),
            metadata.issuer().clone(),
            metadata.authorization_endpoint().clone(),
            metadata.token_endpoint().cloned(),
            metadata.userinfo_endpoint().cloned(),
            metadata.jwks().clone(),
        )
        .set_introspection_uri(IntrospectionUrl::from_url(
            metadata
                .additional_metadata()
                .introspection_endpoint
                .clone(),
        ));

        Ok(ProviderClient { metadata, client })
    }
}
