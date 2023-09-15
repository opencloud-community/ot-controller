// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! # Job execution system for OpenTalk
//!
//! OpenTalk can execute maintenance jobs, e.g. for cleaning up orphan meetings,
//! limiting retention time of data and similar tasks. This module contains both
//! the system used for executing these jobs as well as their implementations.

#![warn(
    bad_style,
    missing_debug_implementations,
    missing_docs,
    overflowing_literals,
    patterns_in_fns_without_body,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]

pub mod jobs;

mod error;
mod events;

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use database::Db;
use log::Log;
use opentalk_log::{error, info};
use serde_json::json;
use settings::Settings;
use signaling_core::ExchangeHandle;

pub use error::Error;

/// Execute a job
pub async fn execute<J: Job>(
    logger: &dyn Log,
    db: Arc<Db>,
    exchange_handle: ExchangeHandle,
    settings: &Settings,
    parameters: serde_json::Value,
    timeout: Duration,
    hide_duration: bool,
) -> Result<(), Error> {
    let start = Instant::now();

    info!(log: logger, "Starting job execution");
    info!(log: logger, "Loading parameters");

    let parameters = match J::Parameters::try_from_json(parameters) {
        Ok(parameters) => parameters,
        Err(e) => {
            error!(log: logger, "{e}");
            return Err(e);
        }
    };

    match tokio::time::timeout(
        timeout,
        J::execute(logger, db, exchange_handle, settings, parameters),
    )
    .await
    {
        Ok(Ok(())) => {
            info!(log: logger, "");
            info!(log: logger, "Job finished successfully");
            if !hide_duration {
                info!(log: logger, "Duration: {:?}", start.elapsed());
            }
            Ok(())
        }
        Ok(Err(e)) => {
            info!(log: logger, "");
            error!(log: logger, "{}", e);
            info!(log: logger, "Job failed");
            if !hide_duration {
                info!(log: logger, "Duration: {:?}", start.elapsed());
            }
            Err(e)
        }
        Err(e) => {
            info!(log: logger, "");
            let e = Error::from(e);
            error!(log: logger, "{e}");
            info!(log: logger, "Job failed");
            if !hide_duration {
                info!(log: logger, "Duration: {:?}", start.elapsed());
            }
            Err(e)
        }
    }
}

/// A trait for job parameters
pub trait JobParameters: Sized {
    /// Try to load the job parameters from JSON
    fn try_from_json(json: serde_json::Value) -> Result<Self, Error>;

    /// Serialize the job parameters to JSON
    fn to_json(&self) -> serde_json::Value;
}

impl JobParameters for () {
    fn try_from_json(_json: serde_json::Value) -> Result<Self, Error> {
        Ok(())
    }

    fn to_json(&self) -> serde_json::Value {
        json!({})
    }
}

/// A trait for defining jobs that can be executed by the job execution system
#[async_trait]
pub trait Job {
    /// The type of parameters required for executing the job
    type Parameters: JobParameters;

    /// Execute the job
    async fn execute(
        logger: &dyn Log,
        db: Arc<Db>,
        exchange_handle: ExchangeHandle,
        settings: &Settings,
        parameters: Self::Parameters,
    ) -> Result<(), Error>;
}
