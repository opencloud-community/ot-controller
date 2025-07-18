// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use either::Either;
use opentalk_types_common::{time::Timestamp, users::DisplayName};
use opentalk_types_signaling::{ParticipantId, ParticipationKind, Role};
use opentalk_types_signaling_control::state::ControlState;

use crate::{SignalingModuleError, SignalingRoomId, VolatileStorage};

pub mod exchange;
pub mod storage;

pub use opentalk_types_signaling_control::MODULE_ID;

use self::storage::{
    AVATAR_URL, AttributeActions, ControlStorage, ControlStorageParticipantAttributes as _,
    DISPLAY_NAME, HAND_IS_UP, HAND_UPDATED_AT, IS_ROOM_OWNER, JOINED_AT, KIND, LEFT_AT, ROLE,
};

pub trait ControlStorageProvider {
    fn control_storage(&mut self) -> &mut dyn ControlStorage;
}

impl ControlStorageProvider for VolatileStorage {
    fn control_storage(&mut self) -> &mut dyn ControlStorage {
        match self.as_mut() {
            Either::Left(v) => v,
            Either::Right(v) => v,
        }
    }
}

#[async_trait::async_trait(?Send)]
pub trait ControlStateExt: Sized {
    type Error;

    async fn from_storage(
        storage: &mut dyn ControlStorage,
        room_id: SignalingRoomId,
        participant_id: ParticipantId,
    ) -> Result<Self, Self::Error>;
}

#[async_trait::async_trait(?Send)]
impl ControlStateExt for ControlState {
    type Error = SignalingModuleError;

    async fn from_storage(
        storage: &mut dyn ControlStorage,
        room: SignalingRoomId,
        participant: ParticipantId,
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
            Option<DisplayName>,
            Option<Role>,
            Option<String>,
            Option<Timestamp>,
            Option<Timestamp>,
            Option<bool>,
            Option<Timestamp>,
            Option<ParticipationKind>,
            Option<bool>,
        ) = {
            storage
                .bulk_attribute_actions(
                    AttributeActions::new(room, participant)
                        .get_global(DISPLAY_NAME)
                        .get_global(ROLE)
                        .get_local(AVATAR_URL)
                        .get_local(JOINED_AT)
                        .get_local(LEFT_AT)
                        .get_local(HAND_IS_UP)
                        .get_local(HAND_UPDATED_AT)
                        .get_local(KIND)
                        .get_global(IS_ROOM_OWNER),
                )
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
            display_name: display_name.unwrap_or_else(DisplayName::participant),
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
