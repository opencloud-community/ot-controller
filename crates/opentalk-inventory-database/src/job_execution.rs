// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage::jobs::{
    Job, JobExecution, NewJobExecution, NewJobExecutionLog, SerialId, UpdateJobExecution,
};
use opentalk_inventory::{JobExecutionInventory, error::StorageBackendSnafu};
use snafu::ResultExt as _;

use crate::{DatabaseConnection, Result};

#[async_trait::async_trait]
impl JobExecutionInventory for DatabaseConnection {
    async fn get_job(&mut self, job_id: SerialId) -> Result<Job> {
        Job::get(&mut self.inner, job_id)
            .await
            .context(StorageBackendSnafu)
    }

    async fn get_all_jobs(&mut self) -> Result<Vec<Job>> {
        Job::get_all(&mut self.inner)
            .await
            .context(StorageBackendSnafu)
    }

    async fn update_job_execution(
        &mut self,
        job_execution_id: SerialId,
        job_execution: UpdateJobExecution,
    ) -> Result<JobExecution> {
        job_execution
            .apply(&mut self.inner, job_execution_id)
            .await
            .context(StorageBackendSnafu)
    }

    async fn create_job_execution(
        &mut self,
        job_execution: NewJobExecution,
    ) -> Result<JobExecution> {
        job_execution
            .insert(&mut self.inner)
            .await
            .context(StorageBackendSnafu)
    }

    /// Create a new batch of job execution logs.
    async fn create_job_execution_logs(
        &mut self,
        job_execution_logs: &[NewJobExecutionLog],
    ) -> Result<()> {
        NewJobExecutionLog::insert_batch(&mut self.inner, job_execution_logs)
            .await
            .context(StorageBackendSnafu)
    }
}
