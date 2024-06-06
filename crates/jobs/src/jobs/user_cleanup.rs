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
