// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod control_storage;
mod redis;
mod volatile;

pub use control_storage::{AttributeActions, ControlStorage};

// The expiry in seconds for the `skip_waiting_room` key in Redis
const SKIP_WAITING_ROOM_KEY_EXPIRY: u32 = 120;
pub const SKIP_WAITING_ROOM_KEY_REFRESH_INTERVAL: u64 = 60;
// TODO: remove all these re-exports once the functionality is migrated into the ControlStorage trait
pub use redis::room_mutex;

#[cfg(test)]
mod test_common {
    use std::collections::{BTreeMap, BTreeSet};

    use chrono::{TimeZone, Utc};
    use opentalk_db_storage::{
        events::{Event, EventSerialId},
        tariffs::Tariff,
    };
    use opentalk_types::{
        core::{EventId, ParticipantId, RoomId, TariffId, TenantId, Timestamp, UserId},
        signaling::Role,
    };
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

    #[derive(Debug, Clone, Serialize, Deserialize, ToRedisArgs, FromRedisValue, PartialEq, Eq)]
    #[to_redis_args(serde)]
    #[from_redis_value(serde)]
    struct Point {
        x: u32,
        y: u32,
    }

    pub(super) async fn participant_attribute(storage: &mut impl ControlStorage) {
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

    pub(super) async fn participant_attributes(storage: &mut impl ControlStorage) {
        let alice_point = Point { x: 44, y: 55 };
        let bob_point = Point { x: 2, y: 3 };

        storage
            .set_attribute(ROOM, ALICE, "point", alice_point.clone())
            .await
            .unwrap();

        assert_eq!(
            storage
                .get_attribute_for_participants::<Point>(ROOM, "point", &[ALICE, BOB])
                .await
                .unwrap(),
            vec![Some(alice_point.clone()), None]
        );
        assert_eq!(
            storage
                .get_attribute_for_participants::<Point>(ROOM, "point", &[BOB, ALICE])
                .await
                .unwrap(),
            vec![None, Some(alice_point.clone())]
        );

        storage
            .set_attribute(ROOM, BOB, "point", bob_point.clone())
            .await
            .unwrap();

        assert_eq!(
            storage
                .get_attribute_for_participants::<Point>(ROOM, "point", &[BOB, ALICE])
                .await
                .unwrap(),
            vec![Some(bob_point.clone()), Some(alice_point.clone())]
        );

        storage
            .remove_attribute(ROOM, ALICE, "point")
            .await
            .unwrap();
        assert_eq!(
            storage
                .get_attribute_for_participants::<Point>(ROOM, "point", &[BOB, ALICE])
                .await
                .unwrap(),
            vec![Some(bob_point.clone()), None]
        );
    }

    pub(super) async fn participant_remove_attributes(storage: &mut impl ControlStorage) {
        storage
            .set_attribute(ROOM, ALICE, "point", "alice_point")
            .await
            .unwrap();
        storage
            .set_attribute(ROOM, BOB, "point", "bob_point")
            .await
            .unwrap();
        storage
            .set_attribute(ROOM, ALICE, "line", "alice_line")
            .await
            .unwrap();

        assert_eq!(
            storage
                .get_attribute_for_participants::<String>(ROOM, "point", &[ALICE, BOB])
                .await
                .unwrap(),
            vec![
                Some("alice_point".to_string()),
                Some("bob_point".to_string())
            ]
        );
        assert_eq!(
            storage
                .get_attribute_for_participants::<String>(ROOM, "line", &[ALICE, BOB])
                .await
                .unwrap(),
            vec![Some("alice_line".to_string()), None]
        );

        storage.remove_attribute_key(ROOM, "point").await.unwrap();

        assert_eq!(
            storage
                .get_attribute_for_participants::<String>(ROOM, "point", &[ALICE, BOB])
                .await
                .unwrap(),
            vec![None, None]
        );
        assert_eq!(
            storage
                .get_attribute_for_participants::<String>(ROOM, "line", &[ALICE, BOB])
                .await
                .unwrap(),
            vec![Some("alice_line".to_string()), None]
        );
    }

    pub(super) async fn get_role_and_left_for_room_participants(storage: &mut impl ControlStorage) {
        storage.add_participant_to_set(ROOM, ALICE).await.unwrap();
        storage.add_participant_to_set(ROOM, BOB).await.unwrap();
        storage
            .set_attribute(ROOM, ALICE, "role", Role::Guest)
            .await
            .unwrap();
        storage
            .set_attribute(ROOM, BOB, "role", Role::User)
            .await
            .unwrap();

        assert_eq!(
            storage
                .get_role_and_left_at_for_room_participants(ROOM)
                .await
                .unwrap(),
            BTreeMap::from_iter([
                (ALICE, (Some(Role::Guest), None)),
                (BOB, (Some(Role::User), None))
            ])
        );
    }

    pub(super) async fn participant_attributes_bulk(storage: &mut impl ControlStorage) {
        let point = Point { x: 44, y: 55 };

        let results: (Option<String>, Option<String>, Option<String>) = storage
            .bulk_attribute_actions(ROOM, ALICE)
            .get("point")
            .set("point", "alice_point")
            .get("point")
            .del("point")
            .get("point")
            .set("point", point.clone())
            .set("line", "alice_line")
            .apply(storage)
            .await
            .unwrap();

        assert_eq!(results, (None, Some("alice_point".to_string()), None));

        assert_eq!(
            storage
                .get_attribute::<Point>(ROOM, ALICE, "point")
                .await
                .unwrap(),
            point
        );
        assert_eq!(
            storage
                .get_attribute::<String>(ROOM, ALICE, "line")
                .await
                .unwrap(),
            "alice_line".to_string()
        );
    }

    pub(super) async fn tariff(storage: &mut impl ControlStorage) {
        let room_id = RoomId::generate();

        assert!(storage.get_tariff(room_id).await.is_err());

        let tariff_1 = Tariff {
            id: TariffId::generate(),
            name: "Tariff 1".to_string(),
            created_at: Utc.with_ymd_and_hms(2024, 5, 16, 1, 2, 3).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2024, 5, 16, 1, 2, 3).unwrap(),
            quotas: Default::default(),
            disabled_modules: Default::default(),
            disabled_features: Default::default(),
        };

        let tariff_2 = Tariff {
            id: TariffId::generate(),
            name: "Tariff 2".to_string(),
            created_at: Utc.with_ymd_and_hms(2023, 3, 21, 14, 20, 31).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2023, 12, 11, 23, 42, 45).unwrap(),
            quotas: Default::default(),
            disabled_modules: Default::default(),
            disabled_features: Default::default(),
        };

        assert_eq!(
            storage
                .try_init_tariff(room_id, tariff_1.clone())
                .await
                .unwrap(),
            tariff_1
        );

        assert_eq!(storage.get_tariff(room_id).await.unwrap(), tariff_1);

        // Verify that we still get tariff 1 returned when attempting to set to tariff 2 after initialization
        assert_eq!(
            storage
                .try_init_tariff(room_id, tariff_2.clone())
                .await
                .unwrap(),
            tariff_1
        );

        storage.delete_tariff(room_id).await.unwrap();

        assert!(storage.get_tariff(room_id).await.is_err());
    }

    pub(super) async fn event(storage: &mut impl ControlStorage) {
        let room_id = RoomId::generate();

        assert!(storage.get_event(room_id).await.unwrap().is_none());

        let event_1 = Some(Event {
            id: EventId::generate(),
            created_at: Utc.with_ymd_and_hms(2024, 5, 16, 1, 2, 3).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2024, 5, 16, 1, 2, 3).unwrap(),
            id_serial: EventSerialId::from(55i64),
            title: "Event 1".to_string(),
            description: "Event 1 description".to_string(),
            room: room_id,
            created_by: UserId::generate(),
            updated_by: UserId::generate(),
            is_time_independent: true,
            is_all_day: None,
            starts_at: None,
            starts_at_tz: None,
            ends_at: None,
            ends_at_tz: None,
            duration_secs: None,
            is_recurring: None,
            recurrence_pattern: None,
            is_adhoc: true,
            tenant_id: TenantId::generate(),
            revision: 77,
            show_meeting_details: true,
        });

        let event_2 = Some(Event {
            id: EventId::generate(),
            created_at: Utc.with_ymd_and_hms(2021, 2, 2, 1, 2, 3).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2021, 2, 2, 1, 2, 3).unwrap(),
            id_serial: EventSerialId::from(4234i64),
            title: "Event 2".to_string(),
            description: "Event 2 description".to_string(),
            room: room_id,
            created_by: UserId::generate(),
            updated_by: UserId::generate(),
            is_time_independent: true,
            is_all_day: None,
            starts_at: None,
            starts_at_tz: None,
            ends_at: None,
            ends_at_tz: None,
            duration_secs: None,
            is_recurring: None,
            recurrence_pattern: None,
            is_adhoc: true,
            tenant_id: TenantId::generate(),
            revision: 24,
            show_meeting_details: false,
        });

        assert_eq!(
            storage
                .try_init_event(room_id, event_1.clone())
                .await
                .unwrap(),
            event_1
        );

        assert_eq!(storage.get_event(room_id).await.unwrap(), event_1);

        // Verify that we still get event 1 returned when attempting to set to event 2 after initialization
        assert_eq!(
            storage
                .try_init_event(room_id, event_2.clone())
                .await
                .unwrap(),
            event_1
        );

        storage.delete_event(room_id).await.unwrap();

        assert!(storage.get_event(room_id).await.unwrap().is_none());
    }

    pub(super) async fn participant_count(s: &mut impl ControlStorage) {
        let room_id = RoomId::generate();

        assert_eq!(s.get_participant_count(room_id).await.unwrap(), None);

        assert_eq!(s.increment_participant_count(room_id).await.unwrap(), 1);
        assert_eq!(s.get_participant_count(room_id).await.unwrap(), Some(1));
        assert_eq!(s.increment_participant_count(room_id).await.unwrap(), 2);
        assert_eq!(s.get_participant_count(room_id).await.unwrap(), Some(2));
        assert_eq!(s.increment_participant_count(room_id).await.unwrap(), 3);
        assert_eq!(s.get_participant_count(room_id).await.unwrap(), Some(3));
        assert_eq!(s.decrement_participant_count(room_id).await.unwrap(), 2);
        assert_eq!(s.get_participant_count(room_id).await.unwrap(), Some(2));
        assert_eq!(s.decrement_participant_count(room_id).await.unwrap(), 1);
        assert_eq!(s.get_participant_count(room_id).await.unwrap(), Some(1));
        assert_eq!(s.decrement_participant_count(room_id).await.unwrap(), 0);
        assert_eq!(s.get_participant_count(room_id).await.unwrap(), Some(0));

        s.delete_participant_count(room_id).await.unwrap();
        assert_eq!(s.get_participant_count(room_id).await.unwrap(), None);
    }

    pub(super) async fn room_closes_at(s: &mut impl ControlStorage) {
        // redis only deserializes full seconds, therefore we can only compare
        // the values if both values are rounded to seconds
        let closes_at = Timestamp::now().rounded_to_seconds();

        assert_eq!(s.get_room_closes_at(ROOM).await.unwrap(), None);
        s.set_room_closes_at(ROOM, closes_at).await.unwrap();
        assert_eq!(s.get_room_closes_at(ROOM).await.unwrap(), Some(closes_at));
        s.remove_room_closes_at(ROOM).await.unwrap();
        assert_eq!(s.get_room_closes_at(ROOM).await.unwrap(), None);
    }

    pub(super) async fn skip_waiting_room(s: &mut impl ControlStorage) {
        // We can't easily test expiry here because it's fixed to long durations, and we
        // don't want tests to take a long time, they should follow the F.I.R.S.T. principle.

        assert!(!s.get_skip_waiting_room(ALICE).await.unwrap());

        s.set_skip_waiting_room_with_expiry_nx(ALICE, true)
            .await
            .unwrap();

        assert!(s.get_skip_waiting_room(ALICE).await.unwrap());

        s.set_skip_waiting_room_with_expiry(ALICE, false)
            .await
            .unwrap();

        assert!(!s.get_skip_waiting_room(ALICE).await.unwrap());

        s.set_skip_waiting_room_with_expiry(ALICE, true)
            .await
            .unwrap();

        assert!(s.get_skip_waiting_room(ALICE).await.unwrap());

        // Ensure that setting with `nx` doesn't overwrite the existing value
        s.set_skip_waiting_room_with_expiry_nx(ALICE, false)
            .await
            .unwrap();

        assert!(s.get_skip_waiting_room(ALICE).await.unwrap());
    }
}
