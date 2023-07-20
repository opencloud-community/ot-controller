// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Errors that can occur during job execution
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error while loading job parameters
    #[error("Could not load job parameters")]
    ParameterLoading(#[from] serde_json::Error),

    /// The job execution timed out
    #[error("Job timed out")]
    Timeout(#[from] tokio::time::error::Elapsed),
}
