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

use crate::{
    events::{perform_deletion, DeleteSelector},
    Error, Job, JobParameters,
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventCleanupParameters {
    #[serde(default = "default_days_since_last_occurrence")]
    days_since_last_occurrence: u64,

    #[serde(default)]
    fail_on_shared_folder_deletion_error: bool,
}

impl JobParameters for EventCleanupParameters {
    fn try_from_json(json: serde_json::Value) -> Result<Self, Error> {
        serde_json::from_value(json).map_err(Into::into)
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}

/// A job to cleanup events a certain duration after the last occurence
#[derive(Debug)]
pub struct EventCleanup;

#[async_trait]
impl Job for EventCleanup {
    type Parameters = EventCleanupParameters;

    async fn execute(
        logger: &dyn Log,
        db: Arc<Db>,
        exchange_handle: ExchangeHandle,
        settings: &Settings,
        parameters: Self::Parameters,
    ) -> Result<(), Error> {
        info!(log: logger, "Starting data protection cleanup job");
        debug!(log: logger, "Job parameters: {parameters:?}");

        info!(log: logger, "");

        if parameters.days_since_last_occurrence < 1 {
            error!(log: logger, "Number of retention days must be 1 or greater");
            return Err(Error::JobExecutionFailed);
        }

        let now = Utc::now();
        let delete_before =
            match now.checked_sub_days(Days::new(parameters.days_since_last_occurrence)) {
                Some(d) => d,
                None => {
                    error!(log: logger, "Couldn't subtract number of retention days");
                    return Err(Error::JobExecutionFailed);
                }
            };

        if let Err(e) = perform_deletion(
            logger,
            db.clone(),
            exchange_handle,
            settings,
            parameters.fail_on_shared_folder_deletion_error,
            DeleteSelector::ScheduledThatEndedBefore(delete_before),
        )
        .await
        {
            error!(log: logger, "{e:?}");
            return Err(Error::JobExecutionFailed);
        }
        Ok(())
    }
}

fn default_days_since_last_occurrence() -> u64 {
    30
}
