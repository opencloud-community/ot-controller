// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{sync::Arc, time::Duration};

use clap::Subcommand;
use lapin_pool::RabbitMqPool;
use log::Log;
use opentalk_controller_settings::Settings;
use opentalk_database::Db;
use opentalk_jobs::Job;
use opentalk_signaling_core::{ExchangeHandle, ExchangeTask};
use serde::{Deserialize, Serialize};
use serde_json::json;
use snafu::{ResultExt, ensure_whatever};

use self::job_type::JobType;
use crate::Result;

mod job_type;

#[derive(Debug, Serialize, Deserialize)]
struct RawParameters {
    #[serde(flatten)]
    entries: serde_json::Map<String, serde_json::Value>,
}

#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum Command {
    /// Execute a job by its job type id
    Execute {
        /// The type of the job to be executed
        #[clap(value_enum)]
        job_type: JobType,

        /// The parameters that the job uses when executed, encoded in a valid JSON object.
        ///
        /// When not provided, this will be an empty JSON object. That means
        /// each job will fill in its own default parameter object fields. The
        /// default parameters for a job can be queried using the
        /// `jobs default-parameters <JOB_TYPE>` subcommand.
        #[clap(long, default_value = "{}")]
        parameters: String,

        /// Timeout after which the job execution gets aborted, in seconds
        #[clap(long, default_value_t = 3600)]
        timeout: u64,

        /// Don't show the duration it took to run a job. Useful for generating reproducible output
        #[clap(long, default_value_t = false)]
        hide_duration: bool,
    },
    /// Show the default parameter set for a job
    DefaultParameters {
        /// The type of the job for which the parameters should be shown
        ///
        /// The parameters are shown in plain pretty-printed JSON
        #[clap(value_enum)]
        job_type: JobType,
    },
}

pub async fn handle_command(settings: &Settings, command: Command) -> Result<()> {
    match command {
        Command::Execute {
            job_type,
            parameters,
            timeout,
            hide_duration,
        } => execute_job(settings, job_type, parameters, timeout, hide_duration).await,
        Command::DefaultParameters { job_type } => show_default_parameters(job_type),
    }
}

async fn execute_job(
    settings: &Settings,
    job_type: JobType,
    parameters: String,
    timeout: u64,
    hide_duration: bool,
) -> Result<()> {
    let db = Arc::new(
        Db::connect(&settings.database).whatever_context("Failed to connect to database")?,
    );

    ensure_whatever!(timeout > 0, "Timeout must be a strictly positive number");
    let timeout = Duration::from_secs(timeout);

    let logger = Logger;

    let parameters: serde_json::Value =
        serde_json::from_str(&parameters).whatever_context("Failed to serialize parameter")?;
    ensure_whatever!(parameters.is_object(), "Parameters must be a JSON object");

    let rabbitmq_pool = settings.rabbit_mq.as_ref().map(|config| {
        RabbitMqPool::from_config(
            &config.url,
            config.min_connections,
            config.max_channels_per_connection,
        )
    });
    let exchange_handle = match (settings.redis.is_some(), &rabbitmq_pool) {
        (true, Some(rabbitmq_pool)) => ExchangeTask::spawn_with_rabbitmq(rabbitmq_pool.clone())
            .await
            .whatever_context("Failed to spawn exchange task")?,
        _ => ExchangeTask::spawn()
            .await
            .whatever_context("Failed to spawn exchange task")?,
    };

    let data = JobExecutionData {
        logger: &logger,
        db,
        exchange_handle,
        settings,
        parameters,
        timeout,
        hide_duration,
    };

    match job_type {
        JobType::SelfCheck => data.execute::<opentalk_jobs::jobs::SelfCheck>().await,
        JobType::EventCleanup => data.execute::<opentalk_jobs::jobs::EventCleanup>().await,
        JobType::UserCleanup => data.execute::<opentalk_jobs::jobs::UserCleanup>().await,
        JobType::AdhocEventCleanup => {
            data.execute::<opentalk_jobs::jobs::AdhocEventCleanup>()
                .await
        }
        JobType::InviteCleanup => data.execute::<opentalk_jobs::jobs::InviteCleanup>().await,
        JobType::SyncStorageFiles => {
            data.execute::<opentalk_jobs::jobs::SyncStorageFiles>()
                .await
        }
        JobType::RoomCleanup => data.execute::<opentalk_jobs::jobs::RoomCleanup>().await,
        JobType::KeycloakAccountSync => {
            data.execute::<opentalk_jobs::jobs::KeycloakAccountSync>()
                .await
        }
    }
    .whatever_context("Failed to execute job")?;

    Ok(())
}

fn show_default_parameters(job_type: JobType) -> Result<()> {
    match job_type {
        JobType::SelfCheck => show_job_type_default_parameters::<opentalk_jobs::jobs::SelfCheck>(),
        JobType::EventCleanup => {
            show_job_type_default_parameters::<opentalk_jobs::jobs::EventCleanup>()
        }
        JobType::UserCleanup => {
            show_job_type_default_parameters::<opentalk_jobs::jobs::UserCleanup>()
        }
        JobType::AdhocEventCleanup => {
            show_job_type_default_parameters::<opentalk_jobs::jobs::AdhocEventCleanup>()
        }
        JobType::InviteCleanup => {
            show_job_type_default_parameters::<opentalk_jobs::jobs::InviteCleanup>()
        }
        JobType::SyncStorageFiles => {
            show_job_type_default_parameters::<opentalk_jobs::jobs::SyncStorageFiles>()
        }
        JobType::RoomCleanup => {
            show_job_type_default_parameters::<opentalk_jobs::jobs::RoomCleanup>()
        }
        JobType::KeycloakAccountSync => {
            show_job_type_default_parameters::<opentalk_jobs::jobs::KeycloakAccountSync>()
        }
    }
}

fn show_job_type_default_parameters<J: Job>() -> Result<()> {
    use opentalk_jobs::JobParameters;
    let parameters = J::Parameters::try_from_json(json!({}))
        .whatever_context("Failed to load parameter from empty object")?;
    println!(
        "{}",
        serde_json::to_string_pretty(
            &parameters
                .to_json()
                .whatever_context("Parameter isn't json serializable")?
        )
        .whatever_context("Failed to serialize parameter")?
    );
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

struct JobExecutionData<'a> {
    logger: &'a dyn Log,
    db: Arc<Db>,
    exchange_handle: ExchangeHandle,
    settings: &'a Settings,
    parameters: serde_json::Value,
    timeout: Duration,
    hide_duration: bool,
}

impl JobExecutionData<'_> {
    async fn execute<J: opentalk_jobs::Job>(self) -> Result<(), opentalk_jobs::Error> {
        opentalk_jobs::execute::<J>(
            self.logger,
            self.db,
            self.exchange_handle,
            self.settings,
            self.parameters,
            self.timeout,
            self.hide_duration,
        )
        .await
    }
}
