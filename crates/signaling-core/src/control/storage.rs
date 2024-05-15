// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod control_storage;
mod redis;
mod volatile;

pub use control_storage::ControlStorage;

// The expiry in seconds for the `skip_waiting_room` key in Redis
const SKIP_WAITING_ROOM_KEY_EXPIRY: u32 = 120;
pub const SKIP_WAITING_ROOM_KEY_REFRESH_INTERVAL: u64 = 60;

// TODO: remove all these re-exports once the functionality is migrated into the ControlStorage trait
pub use redis::{
    decrement_participant_count, delete_event, delete_participant_count, delete_tariff,
    get_attribute_for_participants, get_event, get_participant_count,
    get_role_and_left_at_for_room_participants, get_room_closes_at, get_skip_waiting_room,
    get_tariff, increment_participant_count, participant_id_in_use, participants_all_left,
    remove_attribute_key, remove_room_closes_at, reset_skip_waiting_room_expiry, room_mutex,
    set_room_closes_at, set_skip_waiting_room_with_expiry, set_skip_waiting_room_with_expiry_nx,
    try_init_event, try_init_tariff, AttrPipeline, ParticipantIdRunnerLock,
};

#[cfg(test)]
mod test_common {
    use std::collections::BTreeSet;

    use opentalk_types::core::ParticipantId;
    use pretty_assertions::assert_eq;
    use redis_args::{FromRedisValue, ToRedisArgs};
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::SignalingRoomId;

    pub const ROOM: SignalingRoomId = SignalingRoomId::nil();
    pub const BOB: ParticipantId = ParticipantId::from_u128(0xdeadbeef);
    pub const ALICE: ParticipantId = ParticipantId::from_u128(0xbadcafe);

    pub(super) async fn participant_set(storage: &mut impl ControlStorage) {
        assert!(!storage.participant_set_exists(ROOM).await.unwrap());
        assert!(!storage.participants_contains(ROOM, BOB).await.unwrap());
        assert!(!storage.participants_contains(ROOM, ALICE).await.unwrap());
        assert!(!storage
            .check_participants_exist(ROOM, &[BOB, ALICE])
            .await
            .unwrap());
        assert!(!storage
            .check_participants_exist(ROOM, &[BOB])
            .await
            .unwrap());
        assert!(!storage
            .check_participants_exist(ROOM, &[ALICE])
            .await
            .unwrap());
        assert_eq!(
            storage.get_all_participants(ROOM).await.unwrap(),
            BTreeSet::default()
        );

        storage.add_participant_to_set(ROOM, BOB).await.unwrap();

        assert!(storage.participant_set_exists(ROOM).await.unwrap());
        assert!(storage.participants_contains(ROOM, BOB).await.unwrap());
        assert!(!storage.participants_contains(ROOM, ALICE).await.unwrap());
        assert!(!storage
            .check_participants_exist(ROOM, &[BOB, ALICE])
            .await
            .unwrap());
        assert!(storage
            .check_participants_exist(ROOM, &[BOB])
            .await
            .unwrap());
        assert!(!storage
            .check_participants_exist(ROOM, &[ALICE])
            .await
            .unwrap());
        assert_eq!(
            storage.get_all_participants(ROOM).await.unwrap(),
            BTreeSet::from([BOB])
        );

        storage.add_participant_to_set(ROOM, ALICE).await.unwrap();

        assert!(storage.participant_set_exists(ROOM).await.unwrap());
        assert!(storage.participants_contains(ROOM, BOB).await.unwrap());
        assert!(storage.participants_contains(ROOM, ALICE).await.unwrap());

        assert!(storage
            .check_participants_exist(ROOM, &[BOB, ALICE])
            .await
            .unwrap());
        assert!(storage
            .check_participants_exist(ROOM, &[BOB])
            .await
            .unwrap());
        assert!(storage
            .check_participants_exist(ROOM, &[ALICE])
            .await
            .unwrap());
        assert_eq!(
            storage.get_all_participants(ROOM).await.unwrap(),
            BTreeSet::from([BOB, ALICE])
        );
    }

    pub(super) async fn participant_attribute(storage: &mut impl ControlStorage) {
        #[derive(
            Debug, Clone, Serialize, Deserialize, ToRedisArgs, FromRedisValue, PartialEq, Eq,
        )]
        #[to_redis_args(serde)]
        #[from_redis_value(serde)]
        struct Point {
            x: u32,
            y: u32,
        }

        let point = Point { x: 32, y: 42 };

        storage
            .set_attribute(ROOM, ALICE, "point", point.clone())
            .await
            .unwrap();

        let loaded: Point = storage.get_attribute(ROOM, ALICE, "point").await.unwrap();
        assert_eq!(loaded, point);

        assert!(storage
            .get_attribute::<Point>(ROOM, BOB, "point")
            .await
            .is_err());
        assert!(storage
            .get_attribute::<Point>(ROOM, ALICE, "line")
            .await
            .is_err());

        storage
            .remove_attribute(ROOM, ALICE, "point")
            .await
            .unwrap();
        assert!(storage
            .get_attribute::<Point>(ROOM, ALICE, "point")
            .await
            .is_err());
    }
}
