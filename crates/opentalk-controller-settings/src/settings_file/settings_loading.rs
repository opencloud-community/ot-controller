// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;
use url::Url;

use super::{
    Authz, Avatar, CallIn, ControllerOidcConfiguration, Database, Defaults, Endpoints, Etcd,
    Etherpad, Extensions, FrontendOidcConfiguration, Http, Keycloak, LiveKitSettings, Logging,
    Metrics, MinIO, MonitoringSettings, Oidc, OidcConfiguration, RabbitMqConfig, RedisConfig,
    Reports, SharedFolder, Spacedeck, Stun, SubroomAudio, Tariffs, Tenants, Turn, UserSearch,
    UserSearchConfiguration,
};
use crate::{
    settings_file::{UserSearchBackend, UsersFindBehavior},
    OidcAndUserSearchConfiguration, Result, SettingsError,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SettingsLoading<OIDC> {
    pub database: Database,

    #[serde(default)]
    pub keycloak: Option<Keycloak>,
    #[serde(default)]
    pub oidc: Option<Oidc>,
    #[serde(default)]
    pub user_search: Option<UserSearch>,

    /// The OIDC and user search configuration.
    ///
    /// This configuration is built from the [`Self::oidc`] and [`Self::user_search`]
    /// fields or from the legacy [`Self::keycloak`] field.
    #[serde(skip)]
    pub oidc_and_user_search: OIDC,

    pub http: Http,
    #[serde(default)]
    pub turn: Option<Turn>,
    #[serde(default)]
    pub stun: Option<Stun>,
    #[serde(default)]
    pub redis: Option<RedisConfig>,
    #[serde(default)]
    pub rabbit_mq: RabbitMqConfig,
    #[serde(default)]
    pub logging: Logging,
    #[serde(default)]
    pub authz: Authz,
    #[serde(default)]
    pub avatar: Avatar,
    #[serde(default)]
    pub metrics: Metrics,

    #[serde(default)]
    pub etcd: Option<Etcd>,

    #[serde(default)]
    pub etherpad: Option<Etherpad>,

    #[serde(default)]
    pub spacedeck: Option<Spacedeck>,

    #[serde(default)]
    pub subroom_audio: Option<SubroomAudio>,

    #[serde(default)]
    pub reports: Option<Reports>,

    #[serde(default)]
    pub shared_folder: Option<SharedFolder>,

    #[serde(default)]
    pub call_in: Option<CallIn>,

    #[serde(default)]
    pub defaults: Defaults,

    #[serde(default)]
    pub endpoints: Endpoints,

    pub minio: MinIO,

    #[serde(default)]
    pub monitoring: Option<MonitoringSettings>,

    #[serde(default)]
    pub tenants: Tenants,

    #[serde(default)]
    pub tariffs: Tariffs,

    pub livekit: LiveKitSettings,

    #[serde(flatten)]
    pub extensions: Extensions,
}

impl<OIDC> SettingsLoading<OIDC> {
    /// internal url builder
    fn build_url<I>(base_url: Url, path_segments: I) -> Result<Url>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let err_url = base_url.clone();
        let mut url = base_url;
        url.path_segments_mut()
            .map_err(|_| SettingsError::NotBaseUrl { url: err_url })?
            .extend(path_segments);
        Ok(url)
    }

    /// Builds the effective OIDC and user search configuration, either from the deprecated `[keycloak]` section
    /// and some deprecated `[endpoints]` settings or from the new `[oidc]` and `[user_search]` sections.
    pub(crate) fn build_oidc_and_user_search_configuration(
        &self,
    ) -> Result<OidcAndUserSearchConfiguration> {
        let keycloak = self.keycloak.clone();
        let disable_users_find = self.endpoints.disable_users_find;
        let users_find_use_kc = self.endpoints.users_find_use_kc;
        let oidc = self.oidc.clone();
        let user_search = self.user_search.clone();

        match (
            keycloak,
            disable_users_find,
            users_find_use_kc,
            oidc,
            user_search,
        ) {
            // Only the new OIDC and user search configuration is present
            (None, None, None, Some(oidc), Some(user_search)) => {
                Self::build_new_oidc_and_user_search_configuration(oidc, user_search)
            }
            // Only the legacy OIDC and user search configuration is present
            (Some(keycloak), _, _, None, None) => {
                Self::build_legacy_oidc_and_user_search_configuration(
                    &keycloak,
                    disable_users_find,
                    users_find_use_kc,
                )?
            }
            // The OIDC and user search configuration is inconsistent
            _ => Err(SettingsError::InconsistentOidcAndUserSearchConfig),
        }
    }

    /// Builds the effective OIDC and user search configuration from the new `[oidc]` and `[user_search]` sections.
    fn build_new_oidc_and_user_search_configuration(
        oidc: Oidc,
        user_search: UserSearch,
    ) -> Result<OidcAndUserSearchConfiguration, SettingsError> {
        // Frontend-specific OIDC configuration
        let frontend_auth_base_url = oidc.frontend.authority.unwrap_or(oidc.authority.clone());
        let frontend_client_id = oidc.frontend.client_id.clone();

        // Controller-specific OIDC configuration
        let controller_auth_base_url = oidc.controller.authority.unwrap_or(oidc.authority);
        let controller_client_id = oidc.controller.client_id.clone();
        let controller_client_secret = oidc.controller.client_secret.clone();

        // User search configuration
        let backend = user_search.backend;
        let api_base_url = user_search.api_base_url;
        let user_search_client_id = user_search
            .client_id
            .unwrap_or(controller_client_id.clone());
        let user_search_client_secret = user_search
            .client_secret
            .unwrap_or(controller_client_secret.clone());
        let external_id_user_attribute_name = user_search.external_id_user_attribute_name.clone();
        let users_find_behavior = user_search.users_find_behavior;

        // Assemble the entire effective OIDC and user search configuration
        let frontend = FrontendOidcConfiguration {
            auth_base_url: frontend_auth_base_url,
            client_id: frontend_client_id,
        };
        let controller = ControllerOidcConfiguration {
            auth_base_url: controller_auth_base_url,
            client_id: controller_client_id,
            client_secret: controller_client_secret.clone(),
        };
        let oidc = OidcConfiguration {
            frontend,
            controller,
        };
        let api = UserSearchConfiguration {
            backend,
            api_base_url,
            client_id: user_search_client_id,
            client_secret: user_search_client_secret,
            external_id_user_attribute_name,
            users_find_behavior,
        };
        Ok(OidcAndUserSearchConfiguration {
            oidc,
            user_search: api,
        })
    }

    /// Builds the effective OIDC and user search configuration from the deprecated `[keycloak]` section
    /// and some deprecated `[endpoints]` settings.
    fn build_legacy_oidc_and_user_search_configuration(
        keycloak: &Keycloak,
        disable_users_find: Option<bool>,
        users_find_use_kc: Option<bool>,
    ) -> Result<Result<OidcAndUserSearchConfiguration, SettingsError>, SettingsError> {
        log::warn!(
                    "You are using deprecated OIDC and user search settings. See docs for [oidc] and [user_search] configuration sections."
                );
        // Collect legacy OIDC and user search settings
        let backend = UserSearchBackend::KeycloakWebapi;
        let api_base_url = Self::build_url(
            keycloak.base_url.clone(),
            ["admin", "realms", &keycloak.realm],
        )?;
        let auth_base_url =
            Self::build_url(keycloak.base_url.clone(), ["realms", &keycloak.realm])?;
        let client_id = keycloak.client_id.clone();
        let client_secret = keycloak.client_secret.clone();
        let external_id_user_attribute_name = keycloak.external_id_user_attribute_name.clone();
        let users_find_behavior = match (
            disable_users_find.unwrap_or_default(),
            users_find_use_kc.unwrap_or_default(),
        ) {
            (true, _) => UsersFindBehavior::Disabled,
            (false, false) => UsersFindBehavior::FromDatabase,
            (false, true) => UsersFindBehavior::FromUserSearchBackend,
        };

        // Assemble the entire effective OIDC and user search configuration
        let frontend = FrontendOidcConfiguration {
            auth_base_url: auth_base_url.clone(),
            client_id: client_id.clone(),
        };
        let controller = ControllerOidcConfiguration {
            auth_base_url,
            client_id: client_id.clone(),
            client_secret: client_secret.clone().clone(),
        };
        let oidc = OidcConfiguration {
            frontend,
            controller,
        };
        let api = UserSearchConfiguration {
            backend,
            api_base_url,
            client_id,
            client_secret,
            external_id_user_attribute_name,
            users_find_behavior,
        };
        Ok(Ok(OidcAndUserSearchConfiguration {
            oidc,
            user_search: api,
        }))
    }
}
