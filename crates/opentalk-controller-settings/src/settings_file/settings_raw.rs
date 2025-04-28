// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

use super::{
    Authz, Avatar, CallIn, Database, Defaults, Endpoints, Etcd, Etherpad, Extensions, Http,
    Keycloak, LiveKitSettings, Logging, Metrics, MinIO, MonitoringSettings, Oidc, RabbitMqConfig,
    RedisConfig, Reports, SharedFolder, Spacedeck, Stun, SubroomAudio, Tariffs, Tenants, Turn,
    UserSearch,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SettingsRaw {
    pub(crate) database: Database,

    #[serde(default)]
    pub(crate) keycloak: Option<Keycloak>,

    #[serde(default)]
    pub(crate) oidc: Option<Oidc>,

    #[serde(default)]
    pub(crate) user_search: Option<UserSearch>,

    #[serde(default)]
    pub(crate) http: Option<Http>,

    #[serde(default)]
    pub(crate) turn: Option<Turn>,

    #[serde(default)]
    pub(crate) stun: Option<Stun>,

    #[serde(default)]
    pub(crate) redis: Option<RedisConfig>,

    #[serde(default)]
    pub(crate) rabbit_mq: Option<RabbitMqConfig>,

    #[serde(default)]
    pub(crate) logging: Option<Logging>,

    #[serde(default)]
    pub(crate) authz: Option<Authz>,

    #[serde(default)]
    pub(crate) avatar: Option<Avatar>,

    #[serde(default)]
    pub(crate) metrics: Option<Metrics>,

    #[serde(default)]
    pub(crate) etcd: Option<Etcd>,

    #[serde(default)]
    pub(crate) etherpad: Option<Etherpad>,

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

#[cfg(test)]
pub(crate) fn settings_raw_minimal_example() -> SettingsRaw {
    use openidconnect::{ClientId, ClientSecret};

    use super::{OidcController, OidcFrontend, UserSearchBackend, UsersFindBehavior};

    SettingsRaw {
        database: Database {
            url: "postgres://postgres:password123@localhost:5432/opentalk".to_string(),
            max_connections: None,
        },
        keycloak: None,
        oidc: Some(Oidc {
            authority: "http://localhost:8080/realms/opentalk"
                .parse()
                .expect("must be a valid url"),
            frontend: OidcFrontend {
                authority: None,
                client_id: ClientId::new("Webapp".to_string()),
            },
            controller: OidcController {
                authority: None,
                client_id: ClientId::new("Controller".to_string()),
                client_secret: ClientSecret::new("mysecret".to_string()),
            },
        }),
        user_search: Some(UserSearch {
            backend: UserSearchBackend::KeycloakWebapi(
                crate::settings_file::UserSearchBackendKeycloakWebapi {
                    api_base_url: "http://localhost:8080/admin/realms/opentalk"
                        .parse()
                        .expect("must be a valid url"),
                    client_id: None,
                    client_secret: None,
                    external_id_user_attribute_name: None,
                },
            ),
            users_find_behavior: UsersFindBehavior::Disabled,
        }),
        http: None,
        turn: None,
        stun: None,
        redis: None,
        rabbit_mq: None,
        logging: None,
        authz: None,
        avatar: None,
        metrics: None,
        etcd: None,
        etherpad: None,
        spacedeck: None,
        subroom_audio: None,
        reports: None,
        shared_folder: None,
        call_in: None,
        defaults: Defaults::default(),
        endpoints: Endpoints::default(),
        minio: MinIO {
            uri: "http://localhost:9555"
                .parse()
                .expect("must be a valid url"),
            bucket: "controller".to_string(),
            access_key: "minioadmin".to_string(),
            secret_key: "minioadmin".to_string(),
        },
        monitoring: None,
        tenants: Tenants::default(),
        tariffs: Tariffs::default(),
        livekit: LiveKitSettings {
            public_url: "ws://localhost:7880".to_string(),
            service_url: "http://localhost:7880".to_string(),
            api_key: "devkey".to_string(),
            api_secret: "secret".to_string(),
        },
        extensions: Extensions::default(),
    }
}

#[cfg(test)]
pub(crate) const SETTINGS_RAW_MINIMAL_CONFIG_TOML: &str = r#"
        [database]
        url = "postgres://postgres:password123@localhost:5432/opentalk"

        [minio]
        uri = "http://localhost:9555"
        bucket = "controller"
        access_key = "minioadmin"
        secret_key = "minioadmin"

        [livekit]
        public_url = "ws://localhost:7880"
        service_url = "http://localhost:7880"
        api_key = "devkey"
        api_secret = "secret"

        [oidc]
        authority = "http://localhost:8080/realms/opentalk"

        [oidc.frontend]
        client_id = "Webapp"

        [oidc.controller]
        client_id = "Controller"
        client_secret = "mysecret"

        [user_search]
        backend = "keycloak_webapi"
        api_base_url = "http://localhost:8080/admin/realms/opentalk"
        users_find_behavior = "disabled"
        "#;
