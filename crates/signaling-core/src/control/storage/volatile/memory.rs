// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    time::Duration,
};

use async_trait::async_trait;
use opentalk_db_storage::{events::Event, tariffs::Tariff};
use opentalk_types::{
    core::{ParticipantId, RoomId, Timestamp},
    signaling::Role,
};
use redis::{FromRedisValue, ToRedisArgs};
use snafu::{OptionExt as _, ResultExt as _};

use crate::{
    control::storage::{AttributeActions, SKIP_WAITING_ROOM_KEY_EXPIRY},
    ExpiringDataHashMap, NotFoundSnafu, RedisSnafu, SignalingModuleError, SignalingRoomId,
};

type AttributeMap = HashMap<(ParticipantId, String), Vec<Vec<u8>>>;

#[derive(Debug, Clone, Default)]
pub(super) struct MemoryControlState {
    room_participants: HashMap<SignalingRoomId, BTreeSet<ParticipantId>>,
    participant_attributes: HashMap<SignalingRoomId, AttributeMap>,
    room_tariffs: HashMap<RoomId, Tariff>,
    room_events: HashMap<RoomId, Option<Event>>,
    participant_count: HashMap<RoomId, isize>,
    rooms_close_at: HashMap<SignalingRoomId, Timestamp>,
    participants_skip_waiting_room: ExpiringDataHashMap<ParticipantId, bool>,
}

pub struct VolatileStaticMemoryAttributeActions {
    room: SignalingRoomId,
    participant: ParticipantId,
    actions: Vec<AttributeAction>,
}

impl VolatileStaticMemoryAttributeActions {
    pub(super) fn new(room: SignalingRoomId, participant: ParticipantId) -> Self {
        Self {
            room,
            participant,
            actions: Vec::new(),
        }
    }
}

#[derive(Debug)]
enum AttributeAction {
    Set { name: String, value: Vec<Vec<u8>> },
    Get { name: String },
    Delete { name: String },
}

#[async_trait(?Send)]
impl AttributeActions for VolatileStaticMemoryAttributeActions {
    fn set<V: ToRedisArgs>(&mut self, name: &str, value: V) -> &mut Self {
        self.actions.push(AttributeAction::Set {
            name: name.to_string(),
            // to_redis_args will always contain at least one element
            value: value.to_redis_args(),
        });
        self
    }

    fn get(&mut self, name: &str) -> &mut Self {
        self.actions.push(AttributeAction::Get {
            name: name.to_string(),
        });
        self
    }

    fn del(&mut self, name: &str) -> &mut Self {
        self.actions.push(AttributeAction::Delete {
            name: name.to_string(),
        });
        self
    }
}

impl MemoryControlState {
    #[cfg(test)]
    pub(super) fn reset(&mut self) {
        *self = Default::default();
    }

    pub(super) fn participant_set_exists(&self, room: SignalingRoomId) -> bool {
        self.room_participants.contains_key(&room)
    }

    pub(super) fn get_all_participants(&self, room: SignalingRoomId) -> BTreeSet<ParticipantId> {
        self.room_participants
            .get(&room)
            .cloned()
            .unwrap_or_default()
    }

    pub(super) fn remove_participant_set(&mut self, room: SignalingRoomId) {
        self.room_participants.remove(&room);
    }

    pub(super) fn participants_contains(
        &self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> bool {
        self.room_participants
            .get(&room)
            .map(|p| p.contains(&participant))
            .unwrap_or_default()
    }

    pub(super) fn check_participants_exist(
        &self,
        room: SignalingRoomId,
        participants: &[ParticipantId],
    ) -> bool {
        let query_participants = BTreeSet::from_iter(participants.iter().cloned());

        self.room_participants
            .get(&room)
            .map(|p| p.is_superset(&query_participants))
            .unwrap_or_default()
    }

    /// Returns `true` if the participant was added
    pub(super) fn add_participant_to_set(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> bool {
        self.room_participants
            .entry(room)
            .or_default()
            .insert(participant)
    }

    pub(super) fn get_attribute<V>(
        &self,
        room: SignalingRoomId,
        participant: ParticipantId,
        name: &str,
    ) -> Result<Option<V>, SignalingModuleError>
    where
        V: FromRedisValue,
    {
        self.get_attribute_raw(room, participant, name)
            .map(|b| V::from_redis_value(&redis::Value::load_from_redis_u8_vec_vec(b)))
            .transpose()
            .with_context(|_| RedisSnafu {
                message: format!("Failed to get attribute {name}"),
            })
    }

    fn get_attribute_raw(
        &self,
        room: SignalingRoomId,
        participant: ParticipantId,
        name: &str,
    ) -> Option<&Vec<Vec<u8>>> {
        self.participant_attributes
            .get(&room)
            .and_then(|p| p.get(&(participant, name.to_string())))
    }

    fn get_attribute_raw_redis_value(
        &self,
        room: SignalingRoomId,
        participant: ParticipantId,
        name: &str,
    ) -> redis::Value {
        self.get_attribute_raw(room, participant, name)
            .map(|v| redis::Value::load_from_redis_u8_vec_vec(v))
            .unwrap_or(redis::Value::Nil)
    }

    pub(super) fn set_attribute<V>(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        name: &str,
        value: V,
    ) where
        V: core::fmt::Debug + ToRedisArgs + Send + Sync,
    {
        self.set_attribute_raw(room, participant, name, value.to_redis_args());
    }

    fn set_attribute_raw(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        name: &str,
        value: Vec<Vec<u8>>,
    ) {
        self.participant_attributes
            .entry(room)
            .or_default()
            .insert((participant, name.to_string()), value);
    }

    pub(super) fn remove_attribute(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        name: &str,
    ) {
        let is_empty = self
            .participant_attributes
            .get_mut(&room)
            .map(|a| {
                a.remove(&(participant, name.to_string()));
                a.is_empty()
            })
            .unwrap_or_default();
        if is_empty {
            self.participant_attributes.remove(&room);
        }
    }

    pub(super) fn perform_bulk_attribute_actions<T: FromRedisValue>(
        &mut self,
        actions: &VolatileStaticMemoryAttributeActions,
    ) -> Result<T, SignalingModuleError> {
        let mut response = None;

        for action in &actions.actions {
            match action {
                AttributeAction::Set { name, value } => {
                    self.set_attribute_raw(actions.room, actions.participant, name, value.clone());
                }
                AttributeAction::Get { name } => {
                    let value =
                        self.get_attribute_raw_redis_value(actions.room, actions.participant, name);

                    response = match response {
                        None => Some(value),
                        Some(redis::Value::Bulk(mut b)) => {
                            b.push(value);
                            Some(redis::Value::Bulk(b))
                        }
                        Some(v) => Some(redis::Value::Bulk(vec![v, value])),
                    }
                }
                AttributeAction::Delete { name } => {
                    self.remove_attribute(actions.room, actions.participant, name);
                }
            }
        }

        T::from_redis_value(&response.unwrap_or(redis::Value::Nil)).with_context(|e| RedisSnafu {
            message: format!("Failed to read result from bulk attribute actions, {e}"),
        })
    }

    pub(super) fn get_attribute_for_participants<V>(
        &self,
        room: SignalingRoomId,
        name: &str,
        participants: &[ParticipantId],
    ) -> Result<Vec<Option<V>>, SignalingModuleError>
    where
        V: FromRedisValue,
    {
        participants
            .iter()
            .map(|p| self.get_attribute::<V>(room, *p, name))
            .collect()
    }

    pub(super) fn remove_attribute_key(&mut self, room: SignalingRoomId, name: &str) {
        if let Some(attributes) = self.participant_attributes.get_mut(&room) {
            attributes.retain(|k, _v| k.1 != name)
        };
    }

    #[allow(clippy::type_complexity)]
    pub(super) fn get_role_and_left_at_for_room_participants(
        &self,
        room: SignalingRoomId,
    ) -> Result<BTreeMap<ParticipantId, (Option<Role>, Option<Timestamp>)>, SignalingModuleError>
    {
        let participants = Vec::from_iter(self.get_all_participants(room));

        let roles = self.get_attribute_for_participants(room, "role", &participants)?;
        let left_at = self.get_attribute_for_participants(room, "left_at", &participants)?;

        Ok(participants
            .into_iter()
            .zip(std::iter::zip(roles, left_at))
            .collect())
    }

    pub(super) fn try_init_tariff(&mut self, room_id: RoomId, tariff: Tariff) -> Tariff {
        self.room_tariffs.entry(room_id).or_insert(tariff).clone()
    }

    pub(super) fn get_tariff(&self, room_id: RoomId) -> Result<Tariff, SignalingModuleError> {
        self.room_tariffs
            .get(&room_id)
            .with_context(|| NotFoundSnafu)
            .cloned()
    }

    pub(super) fn delete_tariff(&mut self, room_id: RoomId) {
        self.room_tariffs.remove(&room_id);
    }

    pub(super) fn try_init_event(
        &mut self,
        room_id: RoomId,
        event: Option<Event>,
    ) -> Option<Event> {
        self.room_events.entry(room_id).or_insert(event).clone()
    }

    pub(super) fn get_event(&self, room_id: RoomId) -> Result<Option<Event>, SignalingModuleError> {
        Ok(self.room_events.get(&room_id).cloned().unwrap_or_default())
    }

    pub(super) fn delete_event(&mut self, room_id: RoomId) {
        self.room_events.remove(&room_id);
    }

    pub(super) fn increment_participant_count(&mut self, room_id: RoomId) -> isize {
        let count: &mut isize = self.participant_count.entry(room_id).or_default();
        *count += 1;
        *count
    }

    pub(super) fn decrement_participant_count(&mut self, room_id: RoomId) -> isize {
        let count: &mut isize = self.participant_count.entry(room_id).or_default();
        if *count > 0 {
            *count -= 1;
        }
        *count
    }

    pub(super) fn get_participant_count(&self, room_id: RoomId) -> Option<isize> {
        self.participant_count.get(&room_id).cloned()
    }

    pub(super) fn delete_participant_count(&mut self, room_id: RoomId) {
        self.participant_count.remove(&room_id);
    }

    pub(super) fn set_room_closes_at(&mut self, room: SignalingRoomId, timestamp: Timestamp) {
        self.rooms_close_at.entry(room).or_insert(timestamp);
    }

    pub(super) fn get_room_closes_at(&self, room: SignalingRoomId) -> Option<Timestamp> {
        self.rooms_close_at.get(&room).cloned()
    }

    pub(super) fn remove_room_closes_at(&mut self, room: SignalingRoomId) {
        self.rooms_close_at.remove(&room);
    }

    pub(super) fn set_skip_waiting_room_with_expiry(
        &mut self,
        participant: ParticipantId,
        value: bool,
    ) {
        self.participants_skip_waiting_room.insert_with_expiry(
            participant,
            value,
            Duration::from_secs(SKIP_WAITING_ROOM_KEY_EXPIRY.into()),
        );
    }

    fn cleanup_expired_skip_waiting_room_flags(&mut self) {
        self.participants_skip_waiting_room.cleanup_expired();
    }

    pub(super) fn set_skip_waiting_room_with_expiry_nx(
        &mut self,
        participant: ParticipantId,
        value: bool,
    ) {
        self.cleanup_expired_skip_waiting_room_flags();

        let expires_after = Duration::from_secs(SKIP_WAITING_ROOM_KEY_EXPIRY.into());
        self.participants_skip_waiting_room
            .insert_with_expiry_if_not_exists(participant, value, expires_after);
    }

    pub(super) fn reset_skip_waiting_room_expiry(&mut self, participant: ParticipantId) {
        self.cleanup_expired_skip_waiting_room_flags();

        self.participants_skip_waiting_room.update_expiry(
            &participant,
            Duration::from_secs(SKIP_WAITING_ROOM_KEY_EXPIRY.into()),
        );
    }

    pub(super) fn get_skip_waiting_room(&self, participant: ParticipantId) -> bool {
        self.participants_skip_waiting_room
            .get(&participant)
            .copied()
            .unwrap_or_default()
    }
}

trait FromRedisU8VecVec {
    fn load_from_redis_u8_vec_vec(value: &[Vec<u8>]) -> redis::Value;
}

impl FromRedisU8VecVec for redis::Value {
    fn load_from_redis_u8_vec_vec(value: &[Vec<u8>]) -> redis::Value {
        let loaded = value
            .iter()
            .map(|v| redis::Value::Data(v.clone()))
            .collect::<Vec<_>>();
        if loaded.len() == 1 {
            loaded.into_iter().next().unwrap()
        } else {
            redis::Value::Bulk(loaded)
        }
    }
}

#[cfg(test)]
mod tests {
    use opentalk_types::core::RoomId;
    use pretty_assertions::assert_eq;
    use redis_args::{FromRedisValue, ToRedisArgs};
    use serde::{Deserialize, Serialize};

    use super::*;

    #[test]
    fn roundtrip_attribute() {
        let mut state = MemoryControlState::default();

        #[derive(
            Debug, Clone, Serialize, Deserialize, ToRedisArgs, FromRedisValue, PartialEq, Eq,
        )]
        #[to_redis_args(serde)]
        #[from_redis_value(serde)]
        struct Point {
            x: u32,
            y: u32,
        }

        let room = SignalingRoomId::new_for_room(RoomId::generate());
        let participant = ParticipantId::generate();

        let point = Point { x: 32, y: 42 };

        state.set_attribute(room, participant, "point", point.clone());

        let loaded: Option<Point> = state.get_attribute(room, participant, "point").unwrap();

        assert_eq!(loaded, Some(point));
    }
}
