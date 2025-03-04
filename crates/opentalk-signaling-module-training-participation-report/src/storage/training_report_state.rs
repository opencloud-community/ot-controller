// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, ToRedisArgs, FromRedisValue, Serialize, Deserialize,
)]
#[from_redis_value(serde)]
#[to_redis_args(serde)]
pub(crate) enum TrainingReportState {
    WaitingForParticipant,

    WaitingForInitialTimeout,

    TrackingPresence,
}
