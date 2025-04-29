// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module manages the vote related redis keys.
//!
//! Contains Lua scripts to manipulate multiple redis keys atomically in one request.
//!
//! Each key is defined in its own module with its related functions.

use allowed_tokens::AllowedTokensKey;
use async_trait::async_trait;
use chrono::Utc;
use current_legal_vote_id::CurrentVoteIdKey;
use history::VoteHistoryKey;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types_signaling_legal_vote::vote::LegalVoteId;
use parameters::VoteParametersKey;
use protocol::ProtocolKey;
use snafu::ResultExt;
use vote_count::VoteCountKey;

use super::{LegalVoteParameterStorage as _, LegalVoteStorage, VoteScriptResult, VoteStatus};
use crate::{
    error::{ErrorKind, LegalVoteError},
    storage::protocol::v1::{ProtocolEntry, Vote, VoteEvent},
};

pub(crate) mod allowed_tokens;
pub(crate) mod current_legal_vote_id;
pub(crate) mod history;
pub(crate) mod parameters;
pub mod protocol;
pub(crate) mod vote_count;

#[async_trait(?Send)]
impl LegalVoteStorage for RedisConnection {
    /// End the current vote by moving the vote id to the history & adding a stop/cancel entry
    /// to the vote protocol. See [`END_CURRENT_VOTE_SCRIPT`] for details.
    ///
    /// #Returns
    /// `Ok(true)` when the legal_vote_id was successfully moved to the history
    /// `Ok(false)` when there is no current vote active
    /// `Err(anyhow::Error)` when a redis error occurred
    #[tracing::instrument(name = "legal_vote_end_current_vote", skip(self, end_entry))]
    async fn end_current_vote(
        &mut self,
        room_id: SignalingRoomId,
        legal_vote_id: LegalVoteId,
        end_entry: &ProtocolEntry,
    ) -> Result<bool, SignalingModuleError> {
        redis::Script::new(END_CURRENT_VOTE_SCRIPT)
            .key(CurrentVoteIdKey { room_id })
            .key(ProtocolKey {
                room_id,
                legal_vote_id,
            })
            .key(VoteHistoryKey { room_id })
            .arg(legal_vote_id)
            .arg(end_entry)
            .invoke_async(self)
            .await
            .context(RedisSnafu {
                message: "Failed to end current vote",
            })
    }

    /// Cleanup redis keys related to a vote
    ///
    /// See [`CLEANUP_SCRIPT`] for details.
    ///
    /// Deletes all entries associated with the room & vote id.
    #[tracing::instrument(name = "legal_vote_cleanup_vote", skip(self))]
    async fn cleanup_vote(
        &mut self,
        room_id: SignalingRoomId,
        legal_vote_id: LegalVoteId,
    ) -> Result<(), SignalingModuleError> {
        redis::Script::new(CLEANUP_SCRIPT)
            .key(CurrentVoteIdKey { room_id })
            .key(VoteCountKey {
                room_id,
                legal_vote_id,
            })
            .key(VoteParametersKey {
                room_id,
                legal_vote_id,
            })
            .key(AllowedTokensKey {
                room_id,
                legal_vote_id,
            })
            .key(ProtocolKey {
                room_id,
                legal_vote_id,
            })
            .arg(legal_vote_id)
            .invoke_async(self)
            .await
            .with_context(|_| RedisSnafu {
                message: format!(
                    "Failed to cleanup vote room_id:{} legal_vote_id:{}",
                    room_id, legal_vote_id
                ),
            })
    }
    /// Cast a vote for the specified option
    ///
    /// The vote is done atomically on redis with a Lua script.
    /// See [`VOTE_SCRIPT`] for more details.
    #[tracing::instrument(name = "legal_vote_cast_vote", skip(self, vote_event))]
    async fn vote(
        &mut self,
        room_id: SignalingRoomId,
        legal_vote_id: LegalVoteId,
        vote_event: Vote,
    ) -> Result<VoteScriptResult, LegalVoteError> {
        let token = vote_event.token;
        let parameters =
            self.parameter_get(room_id, legal_vote_id)
                .await?
                .ok_or(LegalVoteError::Vote {
                    source: ErrorKind::InvalidVoteId,
                })?;

        let timestamp = (!parameters.inner.kind.is_hidden()).then(Utc::now);

        let vote_option = vote_event.option;
        let entry = ProtocolEntry::new_with_optional_time(timestamp, VoteEvent::Vote(vote_event));

        redis::Script::new(VOTE_SCRIPT)
            .key(CurrentVoteIdKey { room_id })
            .key(AllowedTokensKey {
                room_id,
                legal_vote_id,
            })
            .key(ProtocolKey {
                room_id,
                legal_vote_id,
            })
            .key(VoteCountKey {
                room_id,
                legal_vote_id,
            })
            .arg(legal_vote_id)
            .arg(token)
            .arg(entry)
            .arg(vote_option)
            .invoke_async(self)
            .await
            .whatever_context::<_, LegalVoteError>("Failed to cast vote")
    }

    async fn get_vote_status(
        &mut self,
        room_id: SignalingRoomId,
        legal_vote_id: LegalVoteId,
    ) -> Result<VoteStatus, SignalingModuleError> {
        redis::Script::new(VOTE_STATUS_SCRIPT)
            .key(CurrentVoteIdKey { room_id })
            .key(VoteHistoryKey { room_id })
            .arg(legal_vote_id)
            .invoke_async(self)
            .await
            .context(RedisSnafu {
                message: "Failed to cast vote",
            })
    }
}

/// Remove the current vote id and add it to the vote history.
/// Adds the provided protocol entry to the corresponding vote protocol.
///
/// The following parameters have to be provided:
///```text
/// KEYS[1] = current vote key
/// KEYS[2] = vote protocol key
/// KEYS[3] = vote history key
///
/// ARGV[1] = vote id
/// ARGV[2] = stop/cancel entry
///```
const END_CURRENT_VOTE_SCRIPT: &str = r#"
if (redis.call("get", KEYS[1]) == ARGV[1]) then
  redis.call("del", KEYS[1])
else
  return 0
end

redis.call("rpush", KEYS[2], ARGV[2])
redis.call("sadd", KEYS[3], ARGV[1])

return 1
"#;

/// Remove all redis entries that are associated with a vote
///
/// The following parameters have to be provided:
/// ```text
/// KEYS[1] = current vote key
/// KEYS[2] = vote count key
/// KEYS[3] = vote parameters key
/// KEYS[4] = allowed users key
/// KEYS[5] = vote protocol key
///
/// ARGV[1] = legal_vote_id
///
/// ```
const CLEANUP_SCRIPT: &str = r#"
if (redis.call("get", KEYS[1]) == ARGV[1]) then
  redis.call("del", KEYS[1])
end

redis.call("del", KEYS[2])
redis.call("del", KEYS[3])
redis.call("del", KEYS[4])
redis.call("del", KEYS[5])
"#;

/// The user allowed token vote script
///
/// Casts a user vote via their token through a Lua script that is executed on redis. The script ensures that the provided `vote id` equals
/// the currently active vote id.
///
/// The voting user's token will be removed from the `allowed tokens list`. This script aborts if the token removal fails.
///
/// When every check succeeds, the `vote count` for the corresponding vote option will be incremented and the provided protocol
/// entry will be pushed to the `protocol`.
///
/// When the token of the voting user is the last allowed token, the return code differs to indicate a [`protocol::Stop::AutoStop`].
///
/// The following parameters have to be provided:
/// ```text
/// ARGV[1] = vote id
/// ARGV[2] = token
/// ARGV[3] = protocol entry
/// ARGV[4] = vote option
///
/// KEYS[1] = current vote key
/// KEYS[2] = allowed tokens key
/// KEYS[3] = protocol key
/// KEYS[4] = vote count key
/// ```
const VOTE_SCRIPT: &str = r#"
if not (redis.call("get", KEYS[1]) == ARGV[1]) then
  return 2
end

if (redis.call("srem", KEYS[2], ARGV[2]) == 1) then
  redis.call("rpush", KEYS[3], ARGV[3])
  redis.call("zincrby", KEYS[4], 1, ARGV[4])
  if (redis.call("scard", KEYS[2]) == 0) then
    return 1
  else
    return 0
  end
else
  return 3
end
"#;

/// Check if the provided vote id is either active, complete or unknown.
///
/// # Returns
/// - 0 when the provide vote id is active
/// - 1 when the provide vote id is complete
/// - 2 when the provide vote id is unknown
///
/// ```text
/// ARGV[1] = vote id
///
/// KEYS[1] = current vote key
/// KEYS[2] = vote history key
/// ```
const VOTE_STATUS_SCRIPT: &str = r#"
if (redis.call("get", KEYS[1]) == ARGV[1]) then
  return 0
elseif (redis.call("SISMEMBER", KEYS[2], ARGV[1]) == 1) then
  return 1
else
  return 2
end
"#;

#[cfg(test)]
pub(crate) mod tests {
    use opentalk_signaling_core::RedisConnection;
    use redis::aio::ConnectionManager;
    use serial_test::serial;

    use crate::storage::test_common;

    async fn storage() -> RedisConnection {
        let redis_url =
            std::env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://0.0.0.0:6379/".to_owned());
        let redis = redis::Client::open(redis_url).expect("Invalid redis url");

        let mut mgr = ConnectionManager::new(redis).await.unwrap();

        redis::cmd("FLUSHALL").exec_async(&mut mgr).await.unwrap();

        RedisConnection::new(mgr)
    }

    #[tokio::test]
    #[serial]
    async fn allow_token() {
        test_common::allow_token(&mut storage().await).await
    }

    #[tokio::test]
    #[serial]
    async fn current_vote() {
        test_common::current_vote(&mut storage().await).await
    }

    #[tokio::test]
    #[serial]
    async fn parameter() {
        test_common::parameter(&mut storage().await).await
    }

    #[tokio::test]
    #[serial]
    async fn voting() {
        test_common::voting(&mut storage().await).await
    }
}
