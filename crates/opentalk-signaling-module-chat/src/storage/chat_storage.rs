// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, HashSet};

use async_trait::async_trait;
use opentalk_signaling_core::{
    SignalingModuleError, SignalingRoomId,
    control::storage::{ControlStorageParticipantAttributesRaw, ControlStorageParticipantSet},
};
use opentalk_types_common::{
    rooms::RoomId,
    time::Timestamp,
    users::{GroupId, GroupName},
};
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_chat::state::StoredMessage;

use crate::ParticipantPair;

#[async_trait(?Send)]
pub(crate) trait ChatStorage:
    ControlStorageParticipantAttributesRaw + ControlStorageParticipantSet
{
    async fn get_room_history(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Vec<StoredMessage>, SignalingModuleError>;

    async fn add_message_to_room_history(
        &mut self,
        room: SignalingRoomId,
        message: &StoredMessage,
    ) -> Result<(), SignalingModuleError>;

    async fn delete_room_history(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError>;

    async fn set_chat_enabled(
        &mut self,
        room: RoomId,
        enabled: bool,
    ) -> Result<(), SignalingModuleError>;

    async fn is_chat_enabled(&mut self, room: RoomId) -> Result<bool, SignalingModuleError>;

    async fn delete_chat_enabled(&mut self, room: RoomId) -> Result<(), SignalingModuleError>;

    async fn set_last_seen_timestamps_private(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        timestamps: &[(ParticipantId, Timestamp)],
    ) -> Result<(), SignalingModuleError>;

    async fn get_last_seen_timestamps_private(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<BTreeMap<ParticipantId, Timestamp>, SignalingModuleError>;

    async fn delete_last_seen_timestamps_private(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError>;

    async fn set_last_seen_timestamps_group(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        timestamps: &[(GroupName, Timestamp)],
    ) -> Result<(), SignalingModuleError>;

    async fn get_last_seen_timestamps_group(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<BTreeMap<GroupName, Timestamp>, SignalingModuleError>;

    async fn delete_last_seen_timestamps_group(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError>;

    async fn set_last_seen_timestamp_global(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        timestamp: Timestamp,
    ) -> Result<(), SignalingModuleError>;

    async fn get_last_seen_timestamp_global(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<Option<Timestamp>, SignalingModuleError>;

    async fn delete_last_seen_timestamp_global(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError>;

    async fn add_private_chat_correspondents(
        &mut self,
        room: SignalingRoomId,
        participant_one: ParticipantId,
        participant_two: ParticipantId,
    ) -> Result<(), SignalingModuleError>;

    async fn delete_private_chat_correspondents(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError>;

    async fn get_private_chat_correspondents(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<HashSet<ParticipantPair>, SignalingModuleError>;

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_private_chat_correspondents_for_participant(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<HashSet<ParticipantId>, SignalingModuleError> {
        Ok(self
            .get_private_chat_correspondents(room)
            .await?
            .into_iter()
            .filter_map(|pair| {
                if pair.participant_one() == participant {
                    Some(pair.participant_two())
                } else if pair.participant_two() == participant {
                    Some(pair.participant_one())
                } else {
                    None
                }
            })
            .collect())
    }

    async fn get_group_chat_history(
        &mut self,
        room: SignalingRoomId,
        group: GroupId,
    ) -> Result<Vec<StoredMessage>, SignalingModuleError>;

    async fn add_message_to_group_chat_history(
        &mut self,
        room: SignalingRoomId,
        group: GroupId,
        message: &StoredMessage,
    ) -> Result<(), SignalingModuleError>;

    async fn delete_group_chat_history(
        &mut self,
        room: SignalingRoomId,
        group: GroupId,
    ) -> Result<(), SignalingModuleError>;

    async fn get_private_chat_history(
        &mut self,
        room: SignalingRoomId,
        participant_one: ParticipantId,
        participant_two: ParticipantId,
    ) -> Result<Vec<StoredMessage>, SignalingModuleError>;

    async fn add_message_to_private_chat_history(
        &mut self,
        room: SignalingRoomId,
        participant_one: ParticipantId,
        participant_two: ParticipantId,
        message: &StoredMessage,
    ) -> Result<(), SignalingModuleError>;

    async fn delete_private_chat_history(
        &mut self,
        room: SignalingRoomId,
        participant_one: ParticipantId,
        participant_two: ParticipantId,
    ) -> Result<(), SignalingModuleError>;

    async fn add_participant_to_group(
        &mut self,
        room: SignalingRoomId,
        group: GroupId,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError>;

    async fn remove_participant_from_group(
        &mut self,
        room: SignalingRoomId,
        group: GroupId,
        participant: ParticipantId,
    );
}
