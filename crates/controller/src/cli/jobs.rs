// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use anyhow::{ensure, Context, Result};
use clap::Subcommand;
use controller_settings::Settings;
use database::Db;
use log::Log;
use serde::{Deserialize, Serialize};
use types::common::jobs::JobType;

#[derive(Debug, Serialize, Deserialize)]
struct RawParameters {
    #[serde(flatten)]
    entries: serde_json::Map<String, serde_json::Value>,
}

#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum Command {
    /// Execute a job by its id number
    Execute {
        /// The type of the job to be executed
        #[clap(value_enum)]
        job_type: JobType,

        /// The parameters that the job uses when executed, encoded in a valid JSON object.
        #[clap(long, default_value = "{}")]
        parameters: String,

        /// Timeout after which the job execution gets aborted, in seconds
        #[clap(long, default_value_t = 3600)]
        timeout: u64,
    },
}

pub async fn handle_command(settings: Settings, command: Command) -> Result<()> {
    match command {
        Command::Execute {
            job_type,
            parameters,
            timeout,
        } => execute_job(settings, job_type, parameters, timeout).await,
    }
}

async fn execute_job(
    settings: Settings,
    job_type: JobType,
    parameters: String,
    timeout: u64,
) -> Result<()> {
    let db = Db::connect(&settings.database).context("Failed to connect to database")?;
    let mut conn = db.get_conn().await?;

    let timeout = Duration::from_secs(u64::try_from(timeout)?);

    let logger = Logger;

    let parameters: serde_json::Value = serde_json::from_str(&parameters)?;
    ensure!(parameters.is_object(), "Parameters must be a JSON object");

    match job_type {
        JobType::SelfCheck => {
            jobs::execute::<jobs::jobs::SelfCheck>(
                &logger, &mut conn, &settings, parameters, timeout,
            )
            .await?;
        }
    }

    Ok(())
}

struct Logger;

impl Log for Logger {
    fn log(&self, record: &log::Record) {
        println!("[{: <5}] {}", record.level().as_str(), record.args());
    }

    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn flush(&self) {}
}
