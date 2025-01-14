// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet};

use async_trait::async_trait;
use opentalk_db_storage::{events::Event, tariffs::Tariff};
use opentalk_types_common::{rooms::RoomId, time::Timestamp};
use opentalk_types_signaling::{ParticipantId, Role};
use opentalk_types_signaling_control::room::CreatorInfo;
use redis::ToRedisArgs;
use redis_args::ToRedisArgs;
use serde::{de::DeserializeOwned, Serialize};
use snafu::ResultExt as _;

use super::LEFT_AT;
use crate::{SerdeJsonSnafu, SignalingModuleError, SignalingRoomId};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    derive_more::Display,
    derive_more::From,
    derive_more::Into,
)]
pub struct GlobalAttributeId(pub &'static str);

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    derive_more::Display,
    derive_more::From,
    derive_more::Into,
)]
pub struct LocalAttributeId(pub &'static str);

#[derive(Debug, Clone, Copy, ToRedisArgs, derive_more::Display)]
#[to_redis_args(fmt = "opentalk-signaling:global-room={room}:participants:attributes:{attribute}")]
#[display("{attribute} in {room}")]
pub struct GlobalRoomAttributeId {
    pub room: RoomId,
    pub attribute: GlobalAttributeId,
}

#[derive(Debug, Clone, Copy, ToRedisArgs, derive_more::Display)]
#[to_redis_args(fmt = "opentalk-signaling:room={room}:participants:attributes:{attribute}")]
#[display("{attribute} in {room}")]
pub struct LocalRoomAttributeId {
    pub room: SignalingRoomId,
    pub attribute: LocalAttributeId,
}

#[derive(Debug, Clone, Copy, derive_more::Display)]
pub enum RoomAttributeId {
    Local(LocalRoomAttributeId),
    Global(GlobalRoomAttributeId),
}

impl ToRedisArgs for RoomAttributeId {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        match self {
            RoomAttributeId::Local(local_room_attribute_id) => {
                local_room_attribute_id.write_redis_args(out)
            }
            RoomAttributeId::Global(global_room_attribute_id) => {
                global_room_attribute_id.write_redis_args(out)
            }
        }
    }
}

impl From<GlobalRoomAttributeId> for RoomAttributeId {
    fn from(attr_id: GlobalRoomAttributeId) -> Self {
        Self::Global(attr_id)
    }
}

impl From<LocalRoomAttributeId> for RoomAttributeId {
    fn from(attr_id: LocalRoomAttributeId) -> Self {
        Self::Local(attr_id)
    }
}

#[derive(Debug)]
pub(crate) enum AttributeAction {
    Set {
        attribute: RoomAttributeId,
        value: serde_json::Value,
    },
    Get {
        attribute: RoomAttributeId,
    },
    Delete {
        attribute: RoomAttributeId,
    },
}

pub struct AttributeActions {
    room: SignalingRoomId,
    participant: ParticipantId,
    actions: Vec<AttributeAction>,
}

impl AttributeActions {
    pub fn new(room: SignalingRoomId, participant: ParticipantId) -> Self {
        Self {
            room,
            participant,
            actions: Vec::new(),
        }
    }

    pub fn set<V: Serialize>(&mut self, attribute: RoomAttributeId, value: V) -> &mut Self {
        let serialized =
            serde_json::to_value(value).expect("attribute value should be serializable");
        self.set_raw(attribute, serialized)
    }

    pub fn set_local<V: Serialize>(&mut self, attribute: LocalAttributeId, value: V) -> &mut Self {
        self.set(
            RoomAttributeId::Local(LocalRoomAttributeId {
                room: self.room,
                attribute,
            }),
            value,
        )
    }

    pub fn set_global<V: Serialize>(
        &mut self,
        attribute: GlobalAttributeId,
        value: V,
    ) -> &mut Self {
        self.set(
            RoomAttributeId::Global(GlobalRoomAttributeId {
                room: self.room.room_id(),
                attribute,
            }),
            value,
        )
    }

    pub fn get(&mut self, attribute: RoomAttributeId) -> &mut Self {
        self.get_raw(attribute)
    }

    pub fn get_local(&mut self, attribute: LocalAttributeId) -> &mut Self {
        self.get(RoomAttributeId::Local(LocalRoomAttributeId {
            room: self.room,
            attribute,
        }))
    }

    pub fn get_global(&mut self, attribute: GlobalAttributeId) -> &mut Self {
        self.get(RoomAttributeId::Global(GlobalRoomAttributeId {
            room: self.room.room_id(),
            attribute,
        }))
    }

    pub fn del(&mut self, attribute: RoomAttributeId) -> &mut Self {
        self.del_raw(attribute)
    }

    pub fn del_local(&mut self, attribute: LocalAttributeId) -> &mut Self {
        self.del(RoomAttributeId::Local(LocalRoomAttributeId {
            room: self.room,
            attribute,
        }))
    }

    pub fn del_global(&mut self, attribute: GlobalAttributeId) -> &mut Self {
        self.del(RoomAttributeId::Global(GlobalRoomAttributeId {
            room: self.room.room_id(),
            attribute,
        }))
    }

    fn set_raw(&mut self, attribute: RoomAttributeId, value: serde_json::Value) -> &mut Self {
        self.actions.push(AttributeAction::Set { attribute, value });
        self
    }

    fn get_raw(&mut self, attribute: RoomAttributeId) -> &mut Self {
        self.actions.push(AttributeAction::Get { attribute });
        self
    }

    fn del_raw(&mut self, attribute: RoomAttributeId) -> &mut Self {
        self.actions.push(AttributeAction::Delete { attribute });
        self
    }

    pub fn participant(&self) -> ParticipantId {
        self.participant
    }

    pub(crate) fn actions(&self) -> &[AttributeAction] {
        &self.actions
    }
}

#[async_trait(?Send)]
pub trait ControlStorage:
    ControlStorageParticipantAttributesRaw
    + ControlStorageEvent
    + ControlStorageParticipantSet
    + ControlStorageSkipWaitingRoom
{
    async fn participants_all_left(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<bool, SignalingModuleError> {
        let participants = self.get_all_participants(room).await?;

        let left_at_attrs: Vec<Option<Timestamp>> = self
            .get_local_attribute_for_participants(&Vec::from_iter(participants), room, LEFT_AT)
            .await?;

        Ok(left_at_attrs.iter().all(Option::is_some))
    }

    async fn remove_attribute_key(
        &mut self,
        attribute: RoomAttributeId,
    ) -> Result<(), SignalingModuleError>;

    async fn get_role_and_left_at_for_room_participants(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<BTreeMap<ParticipantId, (Option<Role>, Option<Timestamp>)>, SignalingModuleError>;

    /// Try to set the active tariff for the room. If the tariff is already set return the current one.
    async fn try_init_tariff(
        &mut self,
        room_id: RoomId,
        tariff: Tariff,
    ) -> Result<Tariff, SignalingModuleError>;

    async fn get_tariff(&mut self, room_id: RoomId) -> Result<Tariff, SignalingModuleError>;

    async fn delete_tariff(&mut self, room_id: RoomId) -> Result<(), SignalingModuleError>;

    async fn increment_participant_count(
        &mut self,
        room_id: RoomId,
    ) -> Result<isize, SignalingModuleError>;

    async fn decrement_participant_count(
        &mut self,
        room_id: RoomId,
    ) -> Result<isize, SignalingModuleError>;

    async fn get_participant_count(
        &mut self,
        room_id: RoomId,
    ) -> Result<Option<isize>, SignalingModuleError>;

    async fn delete_participant_count(
        &mut self,
        room_id: RoomId,
    ) -> Result<(), SignalingModuleError>;

    async fn try_init_creator(
        &mut self,
        room_id: RoomId,
        creator: CreatorInfo,
    ) -> Result<CreatorInfo, SignalingModuleError>;

    async fn get_creator(
        &mut self,
        room_id: RoomId,
    ) -> Result<Option<CreatorInfo>, SignalingModuleError>;

    async fn delete_creator(&mut self, room_id: RoomId) -> Result<(), SignalingModuleError>;

    async fn set_room_closes_at(
        &mut self,
        room: SignalingRoomId,
        timestamp: Timestamp,
    ) -> Result<(), SignalingModuleError>;

    async fn get_room_closes_at(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<Timestamp>, SignalingModuleError>;

    async fn remove_room_closes_at(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError>;
}

#[async_trait(?Send)]
pub trait ControlStorageSkipWaitingRoom {
    /// Set the `skip_waiting_room` flag for participant with an expiry.
    async fn set_skip_waiting_room_with_expiry(
        &mut self,
        participant: ParticipantId,
        value: bool,
    ) -> Result<(), SignalingModuleError>;

    /// Set the `skip_waiting_room` flag for participant with an expiry if the key does not exist.
    async fn set_skip_waiting_room_with_expiry_nx(
        &mut self,
        participant: ParticipantId,
        value: bool,
    ) -> Result<(), SignalingModuleError>;

    /// Extend the `skip_waiting_room` flag for participant with an expiry.
    async fn reset_skip_waiting_room_expiry(
        &mut self,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError>;

    /// Get the `skip_waiting_room` value for participant. If no value is set for the key,
    /// false is returned.
    async fn get_skip_waiting_room(
        &mut self,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError>;
}

#[async_trait(?Send)]
pub trait ControlStorageParticipantSet {
    async fn participant_set_exists(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<bool, SignalingModuleError>;

    async fn get_all_participants(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<BTreeSet<ParticipantId>, SignalingModuleError>;

    async fn remove_participant_set(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<(), SignalingModuleError>;

    async fn participants_contains(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError>;

    async fn check_participants_exist(
        &mut self,
        room: SignalingRoomId,
        participants: &[ParticipantId],
    ) -> Result<bool, SignalingModuleError>;

    /// Returns `true` if the participant id was added, `false` if it already were present
    async fn add_participant_to_set(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<bool, SignalingModuleError>;
}

#[async_trait(?Send)]
pub trait ControlStorageParticipantAttributes: ControlStorageParticipantAttributesRaw {
    async fn get_attribute<V>(
        &mut self,
        participant: ParticipantId,
        attribute: RoomAttributeId,
    ) -> Result<Option<V>, SignalingModuleError>
    where
        V: DeserializeOwned,
    {
        let Some(loaded) = self.get_attribute_raw(participant, attribute).await? else {
            return Ok(None);
        };

        let deserialized = serde_json::from_value(loaded).with_context(|e| SerdeJsonSnafu {
            message: format!(
                "failed to deserialize attribute {attribute} for participant {participant}, {e}"
            ),
        })?;
        Ok(Some(deserialized))
    }

    async fn get_local_attribute<V>(
        &mut self,
        participant: ParticipantId,
        room: SignalingRoomId,
        attribute: LocalAttributeId,
    ) -> Result<Option<V>, SignalingModuleError>
    where
        V: DeserializeOwned,
    {
        self.get_attribute(participant, LocalRoomAttributeId { room, attribute }.into())
            .await
    }

    async fn get_global_attribute<V>(
        &mut self,
        participant: ParticipantId,
        room: RoomId,
        attribute: GlobalAttributeId,
    ) -> Result<Option<V>, SignalingModuleError>
    where
        V: DeserializeOwned,
    {
        self.get_attribute(
            participant,
            GlobalRoomAttributeId { room, attribute }.into(),
        )
        .await
    }

    async fn get_attribute_for_participants<V>(
        &mut self,
        participants: &[ParticipantId],
        attribute: RoomAttributeId,
    ) -> Result<Vec<Option<V>>, SignalingModuleError>
    where
        V: DeserializeOwned,
    {
        let loaded = self
            .get_attribute_for_participants_raw(participants, attribute)
            .await?;

        loaded
            .into_iter()
            .map(|v| v.map(serde_json::from_value).transpose())
            .collect::<Result<Vec<Option<V>>, serde_json::Error>>()
            .with_context(|e| SerdeJsonSnafu{
                message: format!("failed to deserialize attribute {attribute} multiple for participants {participants:?}, {e}")
        })
    }

    async fn get_global_attribute_for_participants<V>(
        &mut self,
        participants: &[ParticipantId],
        room: RoomId,
        attribute: GlobalAttributeId,
    ) -> Result<Vec<Option<V>>, SignalingModuleError>
    where
        V: DeserializeOwned,
    {
        self.get_attribute_for_participants(
            participants,
            GlobalRoomAttributeId { room, attribute }.into(),
        )
        .await
    }

    async fn get_local_attribute_for_participants<V>(
        &mut self,
        participants: &[ParticipantId],
        room: SignalingRoomId,
        attribute: LocalAttributeId,
    ) -> Result<Vec<Option<V>>, SignalingModuleError>
    where
        V: DeserializeOwned,
    {
        self.get_attribute_for_participants(
            participants,
            LocalRoomAttributeId { room, attribute }.into(),
        )
        .await
    }

    async fn set_attribute<V>(
        &mut self,
        participant: ParticipantId,
        attribute: RoomAttributeId,
        value: V,
    ) -> Result<(), SignalingModuleError>
    where
        V: core::fmt::Debug + Serialize + Send + Sync,
    {
        let serialized = serde_json::to_value(value).with_context(|e| SerdeJsonSnafu {
            message: format!(
                "failed to serialize attribute {attribute} for participant {participant}, {e}"
            ),
        })?;
        self.set_attribute_raw(participant, attribute, serialized)
            .await
    }

    async fn set_global_attribute<V>(
        &mut self,
        participant: ParticipantId,
        room: RoomId,
        attribute: GlobalAttributeId,
        value: V,
    ) -> Result<(), SignalingModuleError>
    where
        V: core::fmt::Debug + Serialize + Send + Sync,
    {
        self.set_attribute(
            participant,
            GlobalRoomAttributeId { room, attribute }.into(),
            value,
        )
        .await
    }

    async fn set_local_attribute<V>(
        &mut self,
        participant: ParticipantId,
        room: SignalingRoomId,
        attribute: LocalAttributeId,
        value: V,
    ) -> Result<(), SignalingModuleError>
    where
        V: core::fmt::Debug + Serialize + Send + Sync,
    {
        self.set_attribute(
            participant,
            LocalRoomAttributeId { room, attribute }.into(),
            value,
        )
        .await
    }

    async fn remove_attribute(
        &mut self,
        participant: ParticipantId,
        attribute: RoomAttributeId,
    ) -> Result<(), SignalingModuleError> {
        self.remove_attribute_raw(participant, attribute).await
    }

    async fn remove_global_attribute(
        &mut self,
        participant: ParticipantId,
        room: RoomId,
        attribute: GlobalAttributeId,
    ) -> Result<(), SignalingModuleError> {
        self.remove_attribute(
            participant,
            GlobalRoomAttributeId { room, attribute }.into(),
        )
        .await
    }

    async fn remove_local_attribute(
        &mut self,
        participant: ParticipantId,
        room: SignalingRoomId,
        attribute: LocalAttributeId,
    ) -> Result<(), SignalingModuleError> {
        self.remove_attribute(participant, LocalRoomAttributeId { room, attribute }.into())
            .await
    }

    async fn bulk_attribute_actions<T: DeserializeOwned>(
        &mut self,
        actions: &AttributeActions,
    ) -> Result<T, SignalingModuleError> {
        let value = self.bulk_attribute_actions_raw(actions).await?;

        serde_json::from_value(value).with_context(|e| SerdeJsonSnafu {
            message: format!(
                "Failed to deserialize JSON result from redis bulk attribute actions, {e}"
            ),
        })
    }
}

impl<T: ControlStorageParticipantAttributesRaw + ?Sized> ControlStorageParticipantAttributes for T {}

#[async_trait(?Send)]
pub trait ControlStorageParticipantAttributesRaw {
    async fn get_attribute_raw(
        &mut self,
        participant: ParticipantId,
        attribute: RoomAttributeId,
    ) -> Result<Option<serde_json::Value>, SignalingModuleError>;

    async fn get_attribute_for_participants_raw(
        &mut self,
        participants: &[ParticipantId],
        attribute: RoomAttributeId,
    ) -> Result<Vec<Option<serde_json::Value>>, SignalingModuleError>;

    async fn set_attribute_raw(
        &mut self,
        participant: ParticipantId,
        attribute: RoomAttributeId,
        value: serde_json::Value,
    ) -> Result<(), SignalingModuleError>;

    async fn remove_attribute_raw(
        &mut self,
        participant: ParticipantId,
        attribute: RoomAttributeId,
    ) -> Result<(), SignalingModuleError>;

    async fn bulk_attribute_actions_raw(
        &mut self,
        actions: &AttributeActions,
    ) -> Result<serde_json::Value, SignalingModuleError>;
}

#[async_trait(?Send)]
pub trait ControlStorageEvent {
    /// Try to set the active event for the room. If the event is already set return the current one.
    async fn try_init_event(
        &mut self,
        room_id: RoomId,
        event: Option<Event>,
    ) -> Result<Option<Event>, SignalingModuleError>;

    async fn get_event(&mut self, room_id: RoomId) -> Result<Option<Event>, SignalingModuleError>;

    async fn delete_event(&mut self, room_id: RoomId) -> Result<(), SignalingModuleError>;
}
