// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use database::Db;
use log::Log;
use opentalk_log::{debug, error, info};
use serde::{Deserialize, Serialize};
use settings::Settings;
use signaling_core::ExchangeHandle;

use crate::{
    events::{perform_deletion, DeleteSelector},
    Error, Job, JobParameters,
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
        serde_json::from_value(json).map_err(Into::into)
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
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

        if let Err(e) = perform_deletion(
            logger,
            db.clone(),
            exchange_handle,
            settings,
            parameters.fail_on_shared_folder_deletion_error,
            DeleteSelector::AdHocCreatedBefore(delete_before),
        )
        .await
        {
            error!(log: logger, "{e:?}");
            return Err(Error::JobExecutionFailed);
        }
        Ok(())
    }
}

const fn a_day_in_seconds() -> u32 {
    24 * 60 * 60
}
