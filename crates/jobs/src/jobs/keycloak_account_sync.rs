// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use async_trait::async_trait;
use log::Log;
use opentalk_controller_settings::Settings;
use opentalk_database::Db;
use opentalk_db_storage::users::User;
use opentalk_keycloak_admin::KeycloakAdminClient;
use opentalk_log::info;
use opentalk_signaling_core::ExchangeHandle;

use crate::{events::update_user_accounts, Error, Job};

const KEYCLOAK_USER_LIST_LIMIT: i32 = 100;

#[derive(Debug)]
pub struct KeycloakAccountSync;

#[async_trait]
impl Job for KeycloakAccountSync {
    type Parameters = ();

    async fn execute(
        logger: &dyn Log,
        db: Arc<Db>,
        _exchange_handle: ExchangeHandle,
        settings: &Settings,
        _parameters: Self::Parameters,
    ) -> Result<(), Error> {
        info!(log: logger, "Starting account cleanup job");

        let mut conn = db.get_conn().await?;

        let users = User::get_all(&mut conn).await?;

        let kc_admin_client = KeycloakAdminClient::new(
            settings.keycloak.base_url.clone(),
            settings.keycloak.realm.clone(),
            settings.keycloak.client_id.clone().into(),
            settings.keycloak.client_secret.secret().clone(),
        )?;

        let kc_users_count = kc_admin_client.get_users_count().await?;
        let mut kc_users = vec![];
        let mut offset = 0;

        while offset < kc_users_count {
            let chunk_size = std::cmp::min(KEYCLOAK_USER_LIST_LIMIT, kc_users_count - offset);
            let users_chunk = kc_admin_client.get_users_chunk(offset, chunk_size).await?;
            kc_users.extend(users_chunk);
            offset += chunk_size;
        }

        let user_attribute_name = settings.keycloak.external_id_user_attribute_name.as_deref();

        update_user_accounts(logger, &mut conn, users, kc_users, user_attribute_name).await?;

        Ok(())
    }
}
