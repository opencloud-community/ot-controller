// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use derive_more::{AsRef, Display, From, FromStr, Into};
use diesel::{ExpressionMethods, Identifiable, Insertable, QueryDsl, Queryable};
use diesel_async::RunQueryDsl;
use opentalk_database::{DbConnection, Result};
use opentalk_diesel_newtype::DieselNewtype;
use opentalk_types::sql_enum;
use serde::{Deserialize, Serialize};

use crate::schema::{job_execution_logs, job_executions, jobs};

#[derive(
    AsRef,
    Display,
    From,
    FromStr,
    Into,
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    AsExpression,
    FromSqlRow,
    DieselNewtype,
)]
#[diesel(sql_type = diesel::sql_types::BigInt)]
pub struct SerialId(i64);

#[derive(Debug, Clone, Queryable, Identifiable, PartialEq, Eq)]
pub struct Job {
    pub id: SerialId,
    pub name: String,
    pub kind: JobType,
    pub parameters: serde_json::Value,
    pub timeout_secs: i32,
    pub recurrence: String,
}

impl Job {
    #[tracing::instrument(err, skip_all)]
    pub async fn get(conn: &mut DbConnection, id: SerialId) -> Result<Self> {
        let query = jobs::table.filter(jobs::id.eq(id));

        let job: Job = query.get_result(conn).await?;

        Ok(job)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn get_all(conn: &mut DbConnection) -> Result<Vec<Self>> {
        let query = jobs::table;
        let job = query.load(conn).await?;
        Ok(job)
    }
}

#[derive(Debug, Clone, Queryable, Identifiable, PartialEq, Eq)]
pub struct JobExecution {
    pub id: SerialId,
    pub job_id: SerialId,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub job_status: JobStatus,
}

impl JobExecution {}

#[derive(Debug, Insertable)]
#[diesel(table_name = job_executions)]
pub struct NewJobExecution {
    pub job_id: SerialId,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub job_status: JobStatus,
}

impl NewJobExecution {
    #[tracing::instrument(err, skip_all)]
    pub async fn insert(self, conn: &mut DbConnection) -> Result<JobExecution> {
        let job_execution = self
            .insert_into(job_executions::table)
            .get_result(conn)
            .await?;

        Ok(job_execution)
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = job_executions)]
pub struct UpdateJobExecution {
    pub ended_at: Option<DateTime<Utc>>,
    pub job_status: Option<JobStatus>,
}

impl UpdateJobExecution {
    #[tracing::instrument(err, skip_all)]
    pub async fn apply(self, conn: &mut DbConnection, id: SerialId) -> Result<JobExecution> {
        let target = job_executions::table.filter(job_executions::id.eq(&id));
        let job_execution = diesel::update(target).set(self).get_result(conn).await?;

        Ok(job_execution)
    }
}

#[derive(Debug, Clone, Queryable, Identifiable, PartialEq, Eq)]
pub struct JobExecutionLog {
    pub id: SerialId,
    pub execution_id: SerialId,
    pub logged_at: DateTime<Utc>,
    pub log_level: LogLevel,
    pub log_message: String,
}

impl JobExecutionLog {}

#[derive(Debug, Insertable)]
#[diesel(table_name = job_execution_logs)]
pub struct NewJobExecutionLog {
    pub execution_id: SerialId,
    pub logged_at: DateTime<Utc>,
    pub log_level: LogLevel,
    pub log_message: String,
}

impl NewJobExecutionLog {
    #[tracing::instrument(err, skip_all)]
    pub async fn insert(self, conn: &mut DbConnection) -> Result<JobExecutionLog> {
        let job_execution = self
            .insert_into(job_execution_logs::table)
            .get_result(conn)
            .await?;

        Ok(job_execution)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn insert_batch(conn: &mut DbConnection, batch: &[Self]) -> Result<()> {
        // todo: is there a maximum amount of rows for batch inserts?
        batch
            .insert_into(job_execution_logs::table)
            .execute(conn)
            .await?;

        Ok(())
    }
}

sql_enum!(
    #[derive(PartialEq, Eq, Display)]
    JobType,
    "job_type",
    JobTypeType,
    {
        AdhocEventCleanup = b"adhoc_event_cleanup",
        EventCleanup = b"event_cleanup",
        InviteCleanup = b"invite_cleanup",
        SelfCheck = b"self_check",
    }
);

sql_enum!(
    #[derive(PartialEq, Eq, Display)]
    JobStatus,
    "job_status",
    JobStatusType,
    {
        Started = b"started",
        Succeeded = b"succeeded",
        Failed = b"failed",
    }
);

sql_enum!(
    #[derive(PartialEq, Eq, Display)]
    LogLevel,
    "log_level",
    LogLevelType,
    {
        Trace= b"trace",
        Debug = b"debug",
        Info = b"info",
        Warn = b"warn",
        Error = b"error",
    }
);
