// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Days, Utc};
use log::Log;
use opentalk_controller_settings::Settings;
use opentalk_database::Db;
use opentalk_log::{debug, error, info};
use opentalk_signaling_core::ExchangeHandle;
use serde::{Deserialize, Serialize};
use snafu::{Report, ResultExt};

use crate::{
    error::{ParameterLoadingSnafu, ParameterSerializingSnafu},
    users::{perform_deletion, DeleteSelector},
    Error, Job, JobParameters,
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserCleanupParameters {
    #[serde(default = "default_days_since_user_has_been_disabled")]
    days_since_user_has_been_disabled: u64,

    #[serde(default)]
    fail_on_shared_folder_deletion_error: bool,
}

impl JobParameters for UserCleanupParameters {
    fn try_from_json(json: serde_json::Value) -> Result<Self, Error> {
        serde_json::from_value(json).context(ParameterLoadingSnafu)
    }

    fn to_json(&self) -> Result<serde_json::Value, Error> {
        serde_json::to_value(self).context(ParameterSerializingSnafu)
    }
}

/// A job for cleaning up users that were disabled at minimum a defined duration ago
#[derive(Debug)]
pub struct UserCleanup;

#[async_trait]
impl Job for UserCleanup {
    type Parameters = UserCleanupParameters;

    async fn execute(
        logger: &dyn Log,
        db: Arc<Db>,
        exchange_handle: ExchangeHandle,
        settings: &Settings,
        parameters: Self::Parameters,
    ) -> Result<(), Error> {
        info!(log: logger, "Starting disabled user cleanup job");
        debug!(log: logger, "Job parameters: {parameters:?}");

        info!(log: logger, "");

        let now = Utc::now();
        let delete_before = now
            .checked_sub_days(Days::new(parameters.days_since_user_has_been_disabled))
            .ok_or_else(|| {
                error!(log: logger, "Couldn't subtract number of retention days");
                Error::JobExecutionFailed
            })?;

        perform_deletion(
            logger,
            db.clone(),
            exchange_handle,
            settings,
            parameters.fail_on_shared_folder_deletion_error,
            DeleteSelector::DisabledBefore(delete_before),
        )
        .await
        .map_err(|err| {
            error!(log: logger, "{}", Report::from_error(err));
            Error::JobExecutionFailed
        })?;

        Ok(())
    }
}

fn default_days_since_user_has_been_disabled() -> u64 {
    30
}

#[cfg(test)]
mod test {
    use chrono::{DateTime, Days, Utc};
    use log::logger;
    use opentalk_controller_settings::Settings;
    use opentalk_database::DbConnection;
    use opentalk_db_storage::{
        events::{Event, UpdateEvent},
        users::{UpdateUser, User},
    };
    use opentalk_signaling_core::ExchangeHandle;
    use opentalk_test_util::database::DatabaseContext;
    use opentalk_types_common::{events::EventId, users::UserId};

    use super::{default_days_since_user_has_been_disabled, UserCleanup};
    use crate::{
        jobs::test_utils::{
            create_generic_test_event, create_generic_test_invite, create_generic_test_room,
        },
        Job as _,
    };

    fn init_logger() {
        let mut builder = env_logger::Builder::new();
        builder
            .filter_level(log::LevelFilter::Info)
            .format_timestamp(None)
            .parse_default_env();
        let _ = builder.try_init();
    }

    async fn set_disabled_since(
        conn: &mut DbConnection,
        user_id: UserId,
        since: DateTime<Utc>,
    ) -> User {
        UpdateUser {
            title: None,
            email: None,
            firstname: None,
            lastname: None,
            avatar_url: None,
            phone: None,
            display_name: None,
            language: None,
            id_token_exp: None,
            dashboard_theme: None,
            conference_theme: None,
            tariff_id: None,
            tariff_status: None,
            disabled_since: Some(Some(since)),
        }
        .apply(conn, user_id)
        .await
        .unwrap()
    }

    async fn update_event(conn: &mut DbConnection, user: UserId, event: EventId) -> Event {
        UpdateEvent {
            title: None,
            description: None,
            updated_by: user,
            updated_at: Utc::now(),
            is_time_independent: None,
            is_all_day: None,
            starts_at: None,
            starts_at_tz: None,
            ends_at: None,
            ends_at_tz: None,
            duration_secs: None,
            is_recurring: None,
            recurrence_pattern: None,
            is_adhoc: None,
            show_meeting_details: None,
        }
        .apply(conn, event)
        .await
        .unwrap()
    }

    #[ignore = "minio/s3 storage is required for this test"]
    #[actix_rt::test]
    #[serial_test::serial]
    async fn cleanup_user_with_event_and_invites() {
        init_logger();
        let settings = Settings::load("../../extra/example.toml").unwrap();

        let db_ctx = DatabaseContext::new(false).await;
        let mut conn = db_ctx.db.get_conn().await.unwrap();

        let inviter = db_ctx.create_test_user(0, vec![]).await.unwrap();
        let updated_by = db_ctx.create_test_user(2, vec![]).await.unwrap();

        let room = create_generic_test_room(&mut conn, &inviter).await;
        let event = create_generic_test_event(&mut conn, &inviter).await;
        update_event(&mut conn, updated_by.id, event.id).await;

        create_generic_test_invite(&mut conn, &inviter, Some(&updated_by), &room).await;

        let disabled_since = Utc::now()
            .checked_sub_days(Days::new(default_days_since_user_has_been_disabled() + 1))
            .unwrap();
        let updated_by = set_disabled_since(&mut conn, updated_by.id, disabled_since).await;

        let exchange_handle = ExchangeHandle::dummy();

        // User::get filters disabled users
        let user_exists = User::get_all(&mut conn)
            .await
            .unwrap()
            .iter()
            .any(|u| u.id == updated_by.id);
        assert!(user_exists);

        UserCleanup::execute(
            logger(),
            db_ctx.db.clone(),
            exchange_handle,
            &settings,
            serde_json::from_str("{}").unwrap(),
        )
        .await
        .unwrap();

        let user_exists = User::get_all(&mut conn)
            .await
            .unwrap()
            .iter()
            .any(|u| u.id == updated_by.id);
        assert!(!user_exists, "User was not successfully cleaned up");
    }

    #[ignore = "minio/s3 storage is required for this test"]
    #[actix_rt::test]
    #[serial_test::serial]
    async fn cleanup_user() {
        init_logger();
        let settings = Settings::load("../../extra/example.toml").unwrap();

        let db_ctx = DatabaseContext::new(false).await;
        let mut conn = db_ctx.db.get_conn().await.unwrap();

        let user = db_ctx.create_test_user(0, vec![]).await.unwrap();

        let disabled_since = Utc::now()
            .checked_sub_days(Days::new(default_days_since_user_has_been_disabled() + 1))
            .unwrap();
        let inviter = set_disabled_since(&mut conn, user.id, disabled_since).await;

        let exchange_handle = ExchangeHandle::dummy();

        // User::get filters disabled users
        let user_exists = User::get_all(&mut conn)
            .await
            .unwrap()
            .iter()
            .any(|u| u.id == inviter.id);
        assert!(user_exists);

        UserCleanup::execute(
            logger(),
            db_ctx.db.clone(),
            exchange_handle,
            &settings,
            serde_json::from_str("{}").unwrap(),
        )
        .await
        .unwrap();

        let user_exists = User::get_all(&mut conn)
            .await
            .unwrap()
            .iter()
            .any(|u| u.id == inviter.id);
        assert!(!user_exists, "User was not successfully cleaned up");
    }
}
