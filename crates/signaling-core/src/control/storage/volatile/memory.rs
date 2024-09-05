// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{BTreeSet, HashMap},
    time::Duration,
};

use opentalk_db_storage::{events::Event, tariffs::Tariff};
use opentalk_types_common::{rooms::RoomId, time::Timestamp};
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_control::room::CreatorInfo;
use snafu::OptionExt as _;

use crate::{
    control::storage::{
        control_storage::AttributeAction, AttributeActions, AttributeId,
        SKIP_WAITING_ROOM_KEY_EXPIRY,
    },
    ExpiringDataHashMap, NotFoundSnafu, SignalingModuleError, SignalingRoomId,
};

type AttributeMap = HashMap<(ParticipantId, AttributeId), serde_json::Value>;

#[derive(Debug, Clone, Default)]
pub(super) struct MemoryControlState {
    room_participants: HashMap<SignalingRoomId, BTreeSet<ParticipantId>>,
    participant_attributes: HashMap<SignalingRoomId, AttributeMap>,
    room_tariffs: HashMap<RoomId, Tariff>,
    room_events: HashMap<RoomId, Option<Event>>,
    room_creators: HashMap<RoomId, CreatorInfo>,
    participant_count: HashMap<RoomId, isize>,
    rooms_close_at: HashMap<SignalingRoomId, Timestamp>,
    participants_skip_waiting_room: ExpiringDataHashMap<ParticipantId, bool>,
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

    pub(super) fn get_attribute_raw(
        &self,
        room: SignalingRoomId,
        participant: ParticipantId,
        attribute: AttributeId,
    ) -> Option<serde_json::Value> {
        self.participant_attributes
            .get(&room)
            .and_then(|p| p.get(&(participant, attribute)))
            .cloned()
    }

    pub(super) fn get_attribute_for_participants_raw(
        &self,
        room: SignalingRoomId,
        participants: &[ParticipantId],
        attribute: AttributeId,
    ) -> Vec<Option<serde_json::Value>> {
        participants
            .iter()
            .map(|participant| self.get_attribute_raw(room, *participant, attribute))
            .collect()
    }

    pub(super) fn set_attribute_raw(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        attribute: AttributeId,
        value: serde_json::Value,
    ) {
        self.participant_attributes
            .entry(room)
            .or_default()
            .insert((participant, attribute), value);
    }

    pub(super) fn remove_attribute_raw(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        attribute: AttributeId,
    ) {
        let is_empty = self
            .participant_attributes
            .get_mut(&room)
            .map(|a| {
                a.remove(&(participant, attribute));
                a.is_empty()
            })
            .unwrap_or_default();
        if is_empty {
            self.participant_attributes.remove(&room);
        }
    }

    pub(super) fn perform_bulk_attribute_actions_raw(
        &mut self,
        actions: &AttributeActions,
    ) -> Result<serde_json::Value, SignalingModuleError> {
        let room = actions.room();
        let participant = actions.participant();

        let mut response = None;

        for action in actions.actions() {
            match action {
                AttributeAction::Set { attribute, value } => {
                    self.set_attribute_raw(room, participant, *attribute, value.clone());
                }
                AttributeAction::Get { attribute } => {
                    let value =
                        serde_json::to_value(self.get_attribute_raw(room, participant, *attribute))
                            .expect("Option<Value> is serializable");

                    response = match response {
                        None => Some(value),
                        Some(serde_json::Value::Array(mut values)) => {
                            values.push(value);
                            Some(serde_json::Value::Array(values))
                        }
                        Some(v) => Some(serde_json::Value::Array(vec![v, value])),
                    }
                }
                AttributeAction::Delete { attribute } => {
                    self.remove_attribute_raw(room, participant, *attribute);
                }
            }
        }
        Ok(response.unwrap_or_default())
    }

    pub(super) fn remove_attribute_key(&mut self, room: SignalingRoomId, attribute: AttributeId) {
        if let Some(attributes) = self.participant_attributes.get_mut(&room) {
            attributes.retain(|k, _v| k.1 != attribute)
        };
    }

    pub(super) fn try_init_tariff(&mut self, room_id: RoomId, tariff: Tariff) -> Tariff {
        self.room_tariffs.entry(room_id).or_insert(tariff).clone()
    }

    pub(super) fn get_tariff(&self, room_id: RoomId) -> Result<Tariff, SignalingModuleError> {
        self.room_tariffs
            .get(&room_id)
            .with_context(|| NotFoundSnafu {
                message: format!("No tariff found for room {room_id}"),
            })
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

    pub(super) fn try_init_creator(
        &mut self,
        room_id: RoomId,
        creator: CreatorInfo,
    ) -> CreatorInfo {
        self.room_creators.entry(room_id).or_insert(creator).clone()
    }

    pub(super) fn get_creator(&self, room_id: RoomId) -> Option<CreatorInfo> {
        self.room_creators.get(&room_id).cloned()
    }

    pub(super) fn delete_creator(&mut self, room_id: RoomId) {
        self.room_creators.remove(&room_id);
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use redis_args::{FromRedisValue, ToRedisArgs};
    use serde::{Deserialize, Serialize};

    use super::*;

    const POINT: AttributeId = AttributeId::new("point");

    #[test]
    fn roundtrip_attribute_raw() {
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

        let point = serde_json::to_value(Point { x: 32, y: 42 }).unwrap();

        state.set_attribute_raw(room, participant, POINT, point.clone());

        let loaded = state.get_attribute_raw(room, participant, POINT);

        assert_eq!(loaded, Some(point));
    }
}
