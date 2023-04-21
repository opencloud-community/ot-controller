// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Control Module Stub
//!
//! Actual control 'module' code can be found inside `crate::api::signaling::ws::runner`
use crate::prelude::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use types::core::{ParticipantId, ParticipationKind, Timestamp};

pub mod exchange;
pub mod storage;

pub const NAMESPACE: &str = "control";

/// Control module's FrontendData
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlState {
    pub display_name: String,
    pub role: Role,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    pub participation_kind: ParticipationKind,
    pub hand_is_up: bool,
    pub joined_at: Timestamp,
    pub left_at: Option<Timestamp>,
    pub hand_updated_at: Timestamp,
}

impl ControlState {
    pub async fn from_redis(
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
        ): (
            Option<String>,
            Option<Role>,
            Option<String>,
            Option<Timestamp>,
            Option<Timestamp>,
            Option<bool>,
            Option<Timestamp>,
            Option<ParticipationKind>,
        ) = storage::AttrPipeline::new(room_id, participant_id)
            .get("display_name")
            .get("role")
            .get("avatar_url")
            .get("joined_at")
            .get("left_at")
            .get("hand_is_up")
            .get("hand_updated_at")
            .get("kind")
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
        })
    }
}
