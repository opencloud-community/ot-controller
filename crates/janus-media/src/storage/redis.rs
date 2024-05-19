// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{
    RedisConnection, RedisSnafu, SerdeJsonSnafu, SignalingModuleError, SignalingRoomId,
};
use opentalk_types::{core::ParticipantId, signaling::media::ParticipantMediaState};
use redis::AsyncCommands as _;
use redis_args::ToRedisArgs;
use snafu::ResultExt as _;

use super::{presenter::Presenters, MediaStorage};

#[async_trait(?Send)]
impl MediaStorage for RedisConnection {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_media_state(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<Option<ParticipantMediaState>, SignalingModuleError> {
        let json: Option<Vec<u8>> = self
            .get(ParticipantMediaStateKey { room, participant })
            .await
            .context(RedisSnafu {
                message: "Failed to get media state",
            })?;

        if let Some(json) = json {
            serde_json::from_slice(&json).context(SerdeJsonSnafu {
                message: "Failed to convert json to media state",
            })
        } else {
            Ok(None)
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_media_state(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        participant_media_state: &ParticipantMediaState,
    ) -> Result<(), SignalingModuleError> {
        let json = serde_json::to_vec(&participant_media_state).context(SerdeJsonSnafu {
            message: "Failed to convert media state to json",
        })?;

        self.set(ParticipantMediaStateKey { room, participant }, json)
            .await
            .context(RedisSnafu {
                message: "Failed to get media state",
            })?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_media_state(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        self.del(ParticipantMediaStateKey { room, participant })
            .await
            .context(RedisSnafu {
                message: "Failed to delete media state",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn add_presenter(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        self.sadd(Presenters { room }, participant)
            .await
            .context(RedisSnafu {
                message: "Failed to set presenter",
            })?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn is_presenter(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError> {
        let value: bool = self
            .sismember(Presenters { room }, participant)
            .await
            .context(RedisSnafu {
                message: "Failed to check if participant is presenter",
            })?;

        Ok(value)
    }
}

/// Data related to a module inside a participant
#[derive(ToRedisArgs)]
#[to_redis_args(
    fmt = "opentalk-signaling:room={room}:participant={participant}:namespace=media:state"
)]
struct ParticipantMediaStateKey {
    room: SignalingRoomId,
    participant: ParticipantId,
}

#[cfg(test)]
mod test {
    use redis::aio::ConnectionManager;
    use serial_test::serial;

    use super::{super::test_common, *};

    async fn setup() -> RedisConnection {
        let redis_url =
            std::env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://0.0.0.0:6379/".to_owned());
        let redis = redis::Client::open(redis_url).expect("Invalid redis url");

        let mut mgr = ConnectionManager::new(redis).await.unwrap();

        redis::cmd("FLUSHALL")
            .query_async::<_, ()>(&mut mgr)
            .await
            .unwrap();

        RedisConnection::new(mgr)
    }

    #[tokio::test]
    #[serial]
    async fn media_state() {
        let mut redis_conn = setup().await;
        test_common::media_state(&mut redis_conn).await;
    }
}
