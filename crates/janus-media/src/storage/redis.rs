// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{
    RedisConnection, RedisSnafu, SerdeJsonSnafu, SignalingModuleError, SignalingRoomId,
};
use opentalk_types::{core::ParticipantId, signaling::media::ParticipantMediaState};
use redis::AsyncCommands as _;
use snafu::ResultExt as _;

use super::{participant::ParticipantMediaStateKey, MediaStorage};

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
}
