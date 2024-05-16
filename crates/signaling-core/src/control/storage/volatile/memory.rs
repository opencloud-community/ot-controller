// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet, HashMap};

use opentalk_types::{
    core::{ParticipantId, Timestamp},
    signaling::Role,
};
use redis::{FromRedisValue, ToRedisArgs};
use snafu::ResultExt as _;

use crate::{RedisSnafu, SignalingModuleError, SignalingRoomId};

#[derive(Debug, Clone, Default)]
pub(super) struct MemoryControlState {
    room_participants: HashMap<SignalingRoomId, BTreeSet<ParticipantId>>,
    participant_attributes: HashMap<SignalingRoomId, HashMap<(ParticipantId, String), Vec<u8>>>,
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
        self.participant_attributes
            .get(&room)
            .and_then(|p| p.get(&(participant, name.to_string())))
            .map(|b| V::from_redis_value(&redis::Value::Data(b.to_owned())))
            .transpose()
            .with_context(|_| RedisSnafu {
                message: format!("Failed to get attribute {name}"),
            })
    }

    pub(super) fn set_attribute<V>(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        name: &str,
        value: V,
    ) -> Result<(), SignalingModuleError>
    where
        V: core::fmt::Debug + ToRedisArgs + Send + Sync,
    {
        self.participant_attributes.entry(room).or_default().insert(
            (participant, name.to_string()),
            value.to_redis_args().into_iter().next().unwrap(),
        );
        Ok(())
    }

    pub(super) fn remove_attribute(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        name: &str,
    ) -> Result<(), SignalingModuleError> {
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
        Ok(())
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

        state
            .set_attribute(room, participant, "point", point.clone())
            .unwrap();

        let loaded: Option<Point> = state.get_attribute(room, participant, "point").unwrap();

        assert_eq!(loaded, Some(point));
    }
}
