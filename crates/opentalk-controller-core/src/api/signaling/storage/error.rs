// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_api_v1::error::ApiError;
use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
#[non_exhaustive]
pub enum SignalingStorageError {
    #[snafu(display("Redis error: {message}",))]
    RedisError {
        message: String,
        source: redis::RedisError,
    },

    #[snafu(display("Resumption token could not be refreshed as it was used"))]
    ResumptionTokenAlreadyUsed,

    #[snafu(whatever)]
    Other {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, Some)))]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl From<SignalingStorageError> for ApiError {
    fn from(value: SignalingStorageError) -> Self {
        log::error!("SignalingStorage error: {value}");
        ApiError::internal()
    }
}
