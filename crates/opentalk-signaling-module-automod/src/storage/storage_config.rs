// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_automod::config::Parameter;
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, ToRedisArgs, FromRedisValue)]
#[to_redis_args(serde)]
#[from_redis_value(serde)]
pub struct StorageConfig {
    pub started: DateTime<Utc>,
    pub issued_by: ParticipantId,
    pub parameter: Parameter,
}

impl StorageConfig {
    pub fn new(issued_by: ParticipantId, parameter: Parameter) -> Self {
        Self {
            started: Utc::now(),
            issued_by,
            parameter,
        }
    }
}
