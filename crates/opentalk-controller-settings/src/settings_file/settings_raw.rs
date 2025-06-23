// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

use super::{
    Authz, Avatar, CallIn, Database, Defaults, Endpoints, Etcd, Etherpad, Extensions, Frontend,
    Http, Keycloak, LiveKitSettings, Logging, Metrics, MinIO, MonitoringSettings, Oidc,
    OperatorInformation, RabbitMqConfig, RedisConfig, Reports, SharedFolder, Spacedeck,
    SubroomAudio, Tariffs, Tenants, UserSearch,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SettingsRaw {
    pub(crate) frontend: Frontend,

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
    pub(crate) spacedeck: Option<Spacedeck>,

    #[serde(default)]
    pub(crate) subroom_audio: Option<SubroomAudio>,

    #[serde(default)]
    pub(crate) reports: Option<Reports>,

    #[serde(default)]
    pub(crate) shared_folder: Option<SharedFolder>,

    #[serde(default)]
    pub(crate) call_in: Option<CallIn>,

    #[serde(default)]
    pub(crate) defaults: Option<Defaults>,

    #[serde(default)]
    pub(crate) endpoints: Option<Endpoints>,

    pub(crate) minio: MinIO,

    #[serde(default)]
    pub(crate) monitoring: Option<MonitoringSettings>,

    #[serde(default)]
    pub(crate) tenants: Option<Tenants>,

    #[serde(default)]
    pub(crate) tariffs: Option<Tariffs>,

    pub(crate) livekit: LiveKitSettings,

    #[serde(flatten)]
    pub(crate) extensions: Extensions,

    #[serde(default)]
    pub(crate) operator_information: Option<OperatorInformation>,
}

#[cfg(test)]
pub(crate) fn settings_raw_minimal_example() -> SettingsRaw {
    use openidconnect::{ClientId, ClientSecret};
    use url::Url;

    use super::{OidcController, OidcFrontend};

    SettingsRaw {
        frontend: Frontend {
            base_url: Url::parse("https://example.com").unwrap(),
        },
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
            backend: None,
            users_find_behavior: None,
        }),
        http: None,
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
        defaults: None,
        endpoints: None,
        minio: MinIO {
            uri: "http://localhost:9555"
                .parse()
                .expect("must be a valid url"),
            bucket: "controller".to_string(),
            access_key: "minioadmin".to_string(),
            secret_key: "minioadmin".to_string(),
        },
        monitoring: None,
        tenants: None,
        tariffs: None,
        livekit: LiveKitSettings {
            public_url: "ws://localhost:7880".to_string(),
            service_url: "http://localhost:7880".to_string(),
            api_key: "devkey".to_string(),
            api_secret: "secret".to_string(),
        },
        extensions: Extensions::default(),
        operator_information: None,
    }
}

#[cfg(test)]
pub(crate) const SETTINGS_RAW_MINIMAL_CONFIG_TOML: &str = r#"
        [frontend]
        base_url = "https://example.com"

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
        "#;
