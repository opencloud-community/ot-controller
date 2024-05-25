// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types::{
    core::{ParticipantId, ParticipationKind, Timestamp},
    signaling::{control::state::ControlState, Role},
};

use crate::{RedisConnection, SignalingModuleError, SignalingRoomId};

pub mod exchange;
pub mod storage;

pub use opentalk_types::signaling::control::NAMESPACE;

use self::storage::{
    AttributeActions as _, ControlStorageParticipantAttributesBulk as _, AVATAR_URL, DISPLAY_NAME,
    HAND_IS_UP, HAND_UPDATED_AT, IS_ROOM_OWNER, JOINED_AT, KIND, LEFT_AT, ROLE,
};

#[async_trait::async_trait(?Send)]
pub trait ControlStateExt: Sized {
    type Error;

    async fn from_redis(
        redis_conn: &mut RedisConnection,
        room_id: SignalingRoomId,
        participant_id: ParticipantId,
    ) -> Result<Self, Self::Error>;
}

#[async_trait::async_trait(?Send)]
impl ControlStateExt for ControlState {
    type Error = SignalingModuleError;

    async fn from_redis(
        redis_conn: &mut RedisConnection,
        room_id: SignalingRoomId,
        participant_id: ParticipantId,
    ) -> Result<Self, SignalingModuleError> {
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
        ) = {
            redis_conn
                .bulk_attribute_actions(room_id, participant_id)
                .get(DISPLAY_NAME)
                .get(ROLE)
                .get(AVATAR_URL)
                .get(JOINED_AT)
                .get(LEFT_AT)
                .get(HAND_IS_UP)
                .get(HAND_UPDATED_AT)
                .get(KIND)
                .get(IS_ROOM_OWNER)
                .apply(redis_conn)
                .await
        }?;

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
