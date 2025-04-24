// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use super::{
    oidc_and_user_search_builder::OidcAndUserSearchBuilder, Authz, Database, Http, Logging, Oidc,
    RabbitMq, Redis, Stun, Turn, UserSearchBackend,
};
use crate::{settings_file::UsersFindBehavior, Result, SettingsError, SettingsRaw};

/// The settings used for the OpenTalk controller at runtime
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// The raw settings in the format that was loaded from the configuration file and the environment.
    ///
    /// This is provided as a legacy field until the other fields in this
    /// struct for runtime access are migrated. It will be removed from this
    /// struct once everything is available.
    pub raw: Arc<SettingsRaw>,

    /// The OIDC configuration for OpenTalk.
    pub oidc: Oidc,

    /// The user search backend.
    pub user_search_backend: Option<UserSearchBackend>,

    /// The user search behavior.
    pub users_find_behavior: UsersFindBehavior,

    /// The HTTP service settings.
    pub http: Http,

    /// The database connection settings.
    pub database: Database,

    /// The TURN server settings
    pub turn: Option<Turn>,

    /// The STUN settings.
    pub stun: Option<Stun>,

    /// The redis connection settings.
    pub redis: Option<Redis>,

    /// The RabbitMQ connection settings.
    pub rabbit_mq: Option<RabbitMq>,

    /// The Authz settings.
    pub authz: Authz,

    /// The logging settings.
    pub logging: Logging,
}

impl Settings {
    pub(crate) fn try_reload_from(&mut self, new_raw: SettingsRaw) -> Result<()> {
        let mut current_raw = (*self.raw).clone();

        // reload extensions config
        current_raw.extensions = new_raw.extensions;

        // reload turn settings
        current_raw.turn = new_raw.turn;

        // reload metrics
        current_raw.metrics = new_raw.metrics;

        // reload avatar
        current_raw.avatar = new_raw.avatar;

        // reload call in
        current_raw.call_in = new_raw.call_in;

        self.raw = Arc::new(current_raw);

        Ok(())
    }
}

impl TryFrom<Arc<SettingsRaw>> for Settings {
    type Error = SettingsError;

    fn try_from(raw: Arc<SettingsRaw>) -> Result<Self, Self::Error> {
        let OidcAndUserSearchBuilder {
            oidc,
            user_search_backend,
            users_find_behavior,
        } = OidcAndUserSearchBuilder::load_from_settings_raw(&raw)?;

        let http = raw.http.clone().into();
        let database = raw.database.clone().into();
        let turn = raw.turn.clone().map(Into::into);
        let stun = raw.stun.clone().map(Into::into);
        let redis = raw.redis.clone().map(Into::into);
        let rabbit_mq = raw.rabbit_mq.clone().map(Into::into);
        let authz = Authz::from_settings_file(raw.authz.clone(), rabbit_mq.is_some());
        let logging = raw.logging.clone().map(Into::into).unwrap_or_default();

        Ok(Settings {
            raw,
            oidc,
            user_search_backend,
            users_find_behavior,
            http,
            database,
            turn,
            stun,
            redis,
            rabbit_mq,
            authz,
            logging,
        })
    }
}
