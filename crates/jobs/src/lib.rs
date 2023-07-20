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

use std::time::Duration;

use async_trait::async_trait;
use database::DbConnection;
use log::Log;
use settings::Settings;

pub use error::Error;

/// Execute a job
pub async fn execute<J: Job>(
    logger: &dyn Log,
    conn: &mut DbConnection,
    settings: &Settings,
    parameters: serde_json::Value,
    timeout: Duration,
) -> Result<(), Error> {
    let parameters = J::Parameters::try_from_json(parameters)?;

    tokio::time::timeout(timeout, J::execute(logger, conn, settings, parameters)).await?
}

/// A trait for job parameters
pub trait JobParameters: Sized {
    /// Try to load the job parameters from JSON
    fn try_from_json(json: serde_json::Value) -> Result<Self, Error>;
}

impl JobParameters for () {
    fn try_from_json(_json: serde_json::Value) -> Result<Self, Error> {
        Ok(())
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
        conn: &mut DbConnection,
        settings: &Settings,
        parameters: Self::Parameters,
    ) -> Result<(), Error>;
}
