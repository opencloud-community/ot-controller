// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use async_trait::async_trait;
use opentalk_signaling_core::{RedisConnection, RedisSnafu, SignalingModuleError, SignalingRoomId};
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_subroom_audio::{state::WhisperState, whisper_id::WhisperId};
use redis::AsyncCommands;
use redis_args::ToRedisArgs;
use snafu::ResultExt;

use super::SubroomAudioStorage;

/// One specific whisper group. Contains a list of participants and their whisper state
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:subroom-audio:group:{whisper_id}")]
struct WhisperGroupKey {
    room: SignalingRoomId,
    whisper_id: WhisperId,
}

/// A list of all existing whisper groups
#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:subroom-audio:groups")]
struct WhisperGroupsKey {
    room: SignalingRoomId,
}

#[async_trait(?Send)]
impl SubroomAudioStorage for RedisConnection {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn create_whisper_group(
        &mut self,
        room: SignalingRoomId,
        whisper_id: WhisperId,
        participants: &BTreeMap<ParticipantId, WhisperState>,
    ) -> Result<(), SignalingModuleError> {
        let whisper_participants = participants
            .iter()
            .map(|(participant_id, state)| (*participant_id, *state))
            .collect::<Vec<_>>();

        redis::pipe()
            .atomic()
            .sadd(WhisperGroupsKey { room }, whisper_id)
            .hset_multiple(WhisperGroupKey { room, whisper_id }, &whisper_participants)
            .exec_async(self)
            .await
            .context(RedisSnafu {
                message: "Failed to create whisper group",
            })?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_whisper_group(
        &mut self,
        room: SignalingRoomId,
        whisper_id: WhisperId,
    ) -> Result<BTreeMap<ParticipantId, WhisperState>, SignalingModuleError> {
        let participants: Vec<(ParticipantId, WhisperState)> = self
            .hgetall(WhisperGroupKey { room, whisper_id })
            .await
            .context(RedisSnafu {
                message: "Failed to get participants for whisper group",
            })?;

        Ok(participants.into_iter().collect())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_all_whisper_group_ids(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Vec<WhisperId>, SignalingModuleError> {
        self.smembers(WhisperGroupsKey { room })
            .await
            .context(RedisSnafu {
                message: "Failed to get participants for whisper group",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_whisper_group(
        &mut self,
        room: SignalingRoomId,
        whisper_id: WhisperId,
    ) -> Result<(), SignalingModuleError> {
        redis::pipe()
            .atomic()
            .srem(WhisperGroupsKey { room }, whisper_id)
            .del(WhisperGroupKey { room, whisper_id })
            .exec_async(self)
            .await
            .context(RedisSnafu {
                message: "Failed to delete whisper group",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn add_participants(
        &mut self,
        room: SignalingRoomId,
        whisper_id: WhisperId,
        participants: &BTreeMap<ParticipantId, WhisperState>,
    ) -> Result<(), SignalingModuleError> {
        let whisper_participants = participants
            .iter()
            .map(|(participant_id, state)| (*participant_id, *state))
            .collect::<Vec<_>>();

        self.hset_multiple(WhisperGroupKey { room, whisper_id }, &whisper_participants)
            .await
            .context(RedisSnafu {
                message: "Failed to add participants to whisper group",
            })
    }

    /// Remove a participant from the given whisper group
    ///
    /// Returns true when this group gets deleted due to the last participant being removed
    #[tracing::instrument(level = "debug", skip(self))]
    async fn remove_participant(
        &mut self,
        room: SignalingRoomId,
        whisper_id: WhisperId,
        participant_id: ParticipantId,
    ) -> Result<bool, SignalingModuleError> {
        const REMOVE_SCRIPT: &str = r#"
        redis.call("hdel", KEYS[1], ARGV[1])
        if (redis.call("hlen", KEYS[1]) == 0) then
          redis.call("srem", KEYS[2], ARGV[2])
          return 1
        else
          return 0
        end
        "#;

        let group_deleted = redis::Script::new(REMOVE_SCRIPT)
            .key(WhisperGroupKey { room, whisper_id })
            .key(WhisperGroupsKey { room })
            .arg(participant_id)
            .arg(whisper_id)
            .invoke_async(self)
            .await
            .context(RedisSnafu {
                message: "Failed to remove participant from whisper group",
            })?;

        Ok(group_deleted)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn update_participant_state(
        &mut self,
        room: SignalingRoomId,
        whisper_id: WhisperId,
        participant_id: ParticipantId,
        state: WhisperState,
    ) -> Result<(), SignalingModuleError> {
        let () = self
            .hset(WhisperGroupKey { room, whisper_id }, participant_id, state)
            .await
            .context(RedisSnafu {
                message: "Failed to delete participant from whisper group",
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use opentalk_signaling_core::RedisConnection;
    use redis::aio::ConnectionManager;
    use serial_test::serial;

    use super::super::test_common;

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
    async fn create_group() {
        test_common::create_group(&mut storage().await).await
    }

    #[tokio::test]
    #[serial]
    async fn add_participant() {
        test_common::add_participant(&mut storage().await).await
    }

    #[tokio::test]
    #[serial]
    async fn update_participant() {
        test_common::update_participant(&mut storage().await).await
    }

    #[tokio::test]
    #[serial]
    async fn remove_participant() {
        test_common::remove_participant(&mut storage().await).await
    }

    #[tokio::test]
    #[serial]
    async fn manage_groups() {
        test_common::manage_groups(&mut storage().await).await
    }
}
