// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{
    RedisConnection, RedisSnafu, SerdeJsonSnafu, SignalingModuleError, SignalingRoomId,
};
use opentalk_types::{
    core::{ParticipantId, Timestamp},
    signaling::media::{ParticipantMediaState, ParticipantSpeakingState, SpeakingState},
};
use redis::AsyncCommands as _;
use redis_args::ToRedisArgs;
use snafu::{whatever, ResultExt as _};

use super::MediaStorage;
use crate::mcu::{McuId, MediaSessionKey, PublisherInfo};

#[async_trait]
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
    async fn remove_presenter(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        self.srem(Presenters { room }, participant)
            .await
            .context(RedisSnafu {
                message: "Failed to delete presenter",
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

    #[tracing::instrument(level = "debug", skip(self))]
    async fn clear_presenters(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError> {
        self.del(Presenters { room }).await.context(RedisSnafu {
            message: "Failed to delete presenter key",
        })?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_speaking_state(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        is_speaking: bool,
        updated_at: Timestamp,
    ) -> Result<(), SignalingModuleError> {
        self.set(
            SpeakerKey { room, participant },
            Some(SpeakingState {
                is_speaking,
                updated_at,
            }),
        )
        .await
        .context(RedisSnafu {
            message: "Failed to set speaker state",
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_speaking_state(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<Option<SpeakingState>, SignalingModuleError> {
        self.get(SpeakerKey { room, participant })
            .await
            .context(RedisSnafu {
                message: "Failed to get speaker state",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_speaking_state(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError> {
        self.del(SpeakerKey { room, participant })
            .await
            .context(RedisSnafu {
                message: "Failed to delete speaker state",
            })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete_speaking_state_multiple_participants(
        &mut self,
        room: SignalingRoomId,
        participants: &[ParticipantId],
    ) -> Result<(), SignalingModuleError> {
        for &participant in participants {
            self.delete_speaking_state(room, participant).await?;
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_speaking_state_multiple_participants(
        &mut self,
        room: SignalingRoomId,
        participants: &[ParticipantId],
    ) -> Result<Vec<ParticipantSpeakingState>, SignalingModuleError> {
        let mut participant_speakers = Vec::new();

        for &participant in participants {
            if let Some(speaker) = self.get_speaking_state(room, participant).await? {
                participant_speakers.push(ParticipantSpeakingState {
                    participant,
                    speaker,
                });
            }
        }

        Ok(participant_speakers)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn initialize_mcu_load(
        &mut self,
        mcu_id: McuId,
        index: Option<usize>,
    ) -> Result<(), SignalingModuleError> {
        self.zincr(MCU_LOAD, mcu_load_key(&mcu_id, index), 0)
            .await
            .context(RedisSnafu {
                message: "Failed to initialize handle count",
            })?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_mcus_sorted_by_load(
        &mut self,
    ) -> Result<Vec<(McuId, Option<usize>)>, SignalingModuleError> {
        let ids: Vec<String> =
            self.zrangebyscore(MCU_LOAD, "-inf", "+inf")
                .await
                .context(RedisSnafu {
                    message: "Failed to get mcu ids",
                })?;

        ids.iter().map(String::as_str).map(parse_mcu_load).collect()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn increase_mcu_load(
        &mut self,
        mcu_id: McuId,
        index: Option<usize>,
    ) -> Result<(), SignalingModuleError> {
        self.zincr(MCU_LOAD, mcu_load_key(&mcu_id, index), 1)
            .await
            .context(RedisSnafu {
                message: "Failed to increment handle count",
            })?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn decrease_mcu_load(
        &mut self,
        mcu_id: McuId,
        index: Option<usize>,
    ) -> Result<(), SignalingModuleError> {
        self.zincr(MCU_LOAD, mcu_load_key(&mcu_id, index), -1)
            .await
            .context(RedisSnafu {
                message: "Failed to increment handle count",
            })?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn set_publisher_info(
        &mut self,
        media_session_key: MediaSessionKey,
        info: PublisherInfo,
    ) -> Result<(), SignalingModuleError> {
        self.hset(PUBLISHER_INFO, media_session_key.to_string(), info)
            .await
            .context(RedisSnafu {
                message: "Failed to set publisher info",
            })?;
        Ok(())
    }
}

pub(crate) async fn get_publisher_info(
    redis: &mut RedisConnection,
    media_session_key: MediaSessionKey,
) -> Result<PublisherInfo, SignalingModuleError> {
    let info: PublisherInfo = redis
        .hget(PUBLISHER_INFO, media_session_key.to_string())
        .await
        .with_context(|_| RedisSnafu {
            message: format!("Failed to get mcu id for media session key {media_session_key}",),
        })?;
    Ok(info)
}

pub(crate) async fn delete_publisher_info(
    redis: &mut RedisConnection,
    key: MediaSessionKey,
) -> Result<(), redis::RedisError> {
    redis
        .hdel::<_, _, ()>(PUBLISHER_INFO, key.to_string())
        .await
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

#[derive(ToRedisArgs)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:namespace=media:presenters")]
pub(crate) struct Presenters {
    pub(crate) room: SignalingRoomId,
}

/// Data related to a module inside a participant
#[derive(ToRedisArgs)]
#[to_redis_args(
    fmt = "opentalk-signaling:room={room}:participant={participant}:namespace=media:speaker"
)]
pub(crate) struct SpeakerKey {
    pub(crate) room: SignalingRoomId,
    pub(crate) participant: ParticipantId,
}

/// Redis key of the publisher => McuId/JanusRoomId mapping
///
/// This information is used when creating a subscriber
pub(crate) const PUBLISHER_INFO: &str = "opentalk-signaling:mcu:publishers";

/// Redis key for a sorted set of mcu-clients.
///
/// The score represents the amounts of subscribers on that mcu and is used to choose the least
/// busy mcu for a new publisher.
pub(crate) const MCU_LOAD: &str = "opentalk-signaling:mcu:load";

pub(crate) fn mcu_load_key(mcu_id: &McuId, loop_index: Option<usize>) -> String {
    if let Some(loop_index) = loop_index {
        format!("{}@{}", mcu_id, loop_index)
    } else {
        format!("{}", mcu_id)
    }
}

fn parse_mcu_load(s: &str) -> Result<(McuId, Option<usize>), SignalingModuleError> {
    let (id, index) = if let Some((id, loop_index)) = s.rsplit_once('@') {
        (
            id,
            Some(whatever!(
                loop_index.parse::<usize>(),
                "Failed to parse loop_index"
            )),
        )
    } else {
        (s, None)
    };
    Ok((McuId::from(id.to_string()), index))
}

#[cfg(test)]
mod test {
    use redis::aio::ConnectionManager;
    use serial_test::serial;

    use super::{super::test_common, *};

    async fn storage() -> RedisConnection {
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
        test_common::media_state(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn presenter() {
        test_common::presenter(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn speaking_state() {
        test_common::speaking_state(&mut storage().await).await;
    }

    #[tokio::test]
    #[serial]
    async fn mcu_load() {
        test_common::mcu_load(&mut storage().await).await;
    }
}
