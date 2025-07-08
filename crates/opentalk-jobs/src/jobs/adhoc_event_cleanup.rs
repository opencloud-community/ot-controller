// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use log::Log;
use opentalk_controller_settings::Settings;
use opentalk_database::Db;
use opentalk_log::{debug, error, info};
use opentalk_signaling_core::ExchangeHandle;
use serde::{Deserialize, Serialize};
use snafu::{Report, ResultExt};

use crate::{
    Error, Job, JobParameters,
    error::{ParameterLoadingSnafu, ParameterSerializingSnafu},
    events::{DeleteSelector, perform_deletion},
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdhocEventCleanupParameters {
    #[serde(default = "a_day_in_seconds")]
    seconds_since_creation: u32,

    #[serde(default)]
    fail_on_shared_folder_deletion_error: bool,
}

impl JobParameters for AdhocEventCleanupParameters {
    fn try_from_json(json: serde_json::Value) -> Result<Self, Error> {
        serde_json::from_value(json).context(ParameterLoadingSnafu)
    }

    fn to_json(&self) -> Result<serde_json::Value, Error> {
        serde_json::to_value(self).context(ParameterSerializingSnafu)
    }
}

/// A job to cleanup adhoc events a certain duration after they were created
#[derive(Debug)]
pub struct AdhocEventCleanup;

#[async_trait]
impl Job for AdhocEventCleanup {
    type Parameters = AdhocEventCleanupParameters;

    async fn execute(
        logger: &dyn Log,
        db: Arc<Db>,
        exchange_handle: ExchangeHandle,
        settings: &Settings,
        parameters: Self::Parameters,
    ) -> Result<(), Error> {
        info!(log: logger, "Starting ad-hoc event cleanup job");
        debug!(log: logger, "Job parameters: {parameters:?}");

        let delete_before =
            Utc::now() - Duration::seconds(parameters.seconds_since_creation.into());

        perform_deletion(
            logger,
            db.clone(),
            exchange_handle,
            settings,
            parameters.fail_on_shared_folder_deletion_error,
            DeleteSelector::AdHocCreatedBefore(delete_before),
        )
        .await
        .map_err(|e| {
            error!(log: logger, "{}", Report::from_error(e));
            Error::JobExecutionFailed
        })?;
        Ok(())
    }
}

const fn a_day_in_seconds() -> u32 {
    24 * 60 * 60
}
