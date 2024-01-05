// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use types::{
    core::{ParticipantId, ParticipationKind, Timestamp},
    signaling::{control::state::ControlState, Role},
};

use crate::{RedisConnection, SignalingRoomId};

pub mod exchange;
pub mod storage;

pub use types::signaling::control::NAMESPACE;

#[async_trait::async_trait]
pub trait ControlStateExt: Sized {
    async fn from_redis(
        redis_conn: &mut RedisConnection,
        room_id: SignalingRoomId,
        participant_id: ParticipantId,
    ) -> Result<Self>;
}

#[async_trait::async_trait]
impl ControlStateExt for ControlState {
    async fn from_redis(
        redis_conn: &mut RedisConnection,
        room_id: SignalingRoomId,
        participant_id: ParticipantId,
    ) -> Result<Self> {
        #[allow(clippy::type_complexity)]
        let (
            display_name,
            role,
            avatar_url,
            joined_at,
            left_at,
            hand_is_up,
            hand_updated_at,
            participation_kind,
            is_room_owner,
        ): (
            Option<String>,
            Option<Role>,
            Option<String>,
            Option<Timestamp>,
            Option<Timestamp>,
            Option<bool>,
            Option<Timestamp>,
            Option<ParticipationKind>,
            Option<bool>,
        ) = storage::AttrPipeline::new(room_id, participant_id)
            .get("display_name")
            .get("role")
            .get("avatar_url")
            .get("joined_at")
            .get("left_at")
            .get("hand_is_up")
            .get("hand_updated_at")
            .get("kind")
            .get("is_room_owner")
            .query_async(redis_conn)
            .await?;

        if display_name.is_none()
            || joined_at.is_none()
            || hand_is_up.is_none()
            || hand_updated_at.is_none()
        {
            log::error!("failed to fetch some attribute, using fallback defaults");
        }

        Ok(Self {
            display_name: display_name.unwrap_or_else(|| "Participant".into()),
            role: role.unwrap_or(Role::Guest),
            avatar_url,
            participation_kind: participation_kind.unwrap_or(ParticipationKind::Guest),
            hand_is_up: hand_is_up.unwrap_or_default(),
            hand_updated_at: hand_updated_at.unwrap_or_else(Timestamp::unix_epoch),
            joined_at: joined_at.unwrap_or_else(Timestamp::unix_epoch),
            // no default for left_at. If its not found by error,
            // worst case we have a ghost participant,
            left_at,
            is_room_owner: is_room_owner.unwrap_or_default(),
        })
    }
}
