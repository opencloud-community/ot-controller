// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling state for the `chat` namespace

use crate::core::{ParticipantId, Timestamp};
#[allow(unused_imports)]
use crate::imports::*;

use super::{MessageId, Scope};

/// Private chat history
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PrivateHistory {
    /// Private chat correspondent
    pub correspondent: ParticipantId,

    /// Private chat history
    pub history: Vec<StoredMessage>,
}

/// Message type stores in redis
///
/// This needs to have a inner timestamp.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "redis",
    derive(ToRedisArgs, FromRedisValue),
    to_redis_args(serde),
    from_redis_value(serde)
)]
pub struct StoredMessage {
    /// ID of message
    pub id: MessageId,

    /// ID of the participant who sent the message
    pub source: ParticipantId,

    /// Timestamp of when the message was sent
    pub timestamp: Timestamp,

    /// Content of the message
    pub content: String,

    /// Scope of the message
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub scope: Scope,
}
