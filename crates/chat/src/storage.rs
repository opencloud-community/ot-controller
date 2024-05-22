// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod chat_storage;
mod redis;
mod volatile;

pub(crate) use chat_storage::ChatStorage;

#[cfg(test)]
mod test_common {
    use std::{
        collections::HashMap,
        time::{Duration, SystemTime},
    };

    use chrono::{DateTime, Utc};
    use opentalk_signaling_core::SignalingRoomId;
    use opentalk_types::core::ParticipantId;
    use pretty_assertions::assert_eq;

    use super::*;

    pub const ROOM: SignalingRoomId = SignalingRoomId::nil();
    pub const SELF: ParticipantId = ParticipantId::nil();
    pub const BOB: ParticipantId = ParticipantId::from_u128(0xdeadbeef);
    pub const ALICE: ParticipantId = ParticipantId::from_u128(0xbadcafe);

    fn unix_epoch(secs: u64) -> DateTime<Utc> {
        DateTime::from(SystemTime::UNIX_EPOCH + Duration::from_secs(secs))
    }

    pub(super) async fn last_seen_global(storage: &mut dyn ChatStorage) {
        assert!(storage
            .get_last_seen_timestamp_global(ROOM, SELF)
            .await
            .unwrap()
            .is_none());

        storage
            .set_last_seen_timestamp_global(ROOM, SELF, unix_epoch(1000).into())
            .await
            .unwrap();

        assert_eq!(
            storage
                .get_last_seen_timestamp_global(ROOM, SELF)
                .await
                .unwrap(),
            Some(unix_epoch(1000).into())
        );

        storage
            .delete_last_seen_timestamp_global(ROOM, SELF)
            .await
            .unwrap();

        assert!(storage
            .get_last_seen_timestamp_global(ROOM, SELF)
            .await
            .unwrap()
            .is_none());
    }

    pub(super) async fn last_seen_global_is_personal(storage: &mut dyn ChatStorage) {
        // Set the private last seen timestamps as if BOB and ALICE were the participants in the
        // same room, and ensure this doesn't affect the timestamps of SELF.
        {
            // Set BOB's timestamp
            storage
                .set_last_seen_timestamp_global(ROOM, BOB, unix_epoch(1000).into())
                .await
                .unwrap();
        }
        {
            // Set ALICE's timestamp
            storage
                .set_last_seen_timestamp_global(ROOM, ALICE, unix_epoch(2000).into())
                .await
                .unwrap();
        }

        assert!(storage
            .get_last_seen_timestamp_global(ROOM, SELF)
            .await
            .unwrap()
            .is_none());
    }

    pub(super) async fn last_seen_private(storage: &mut dyn ChatStorage) {
        assert!(storage
            .get_last_seen_timestamps_private(ROOM, SELF)
            .await
            .unwrap()
            .is_empty(),);

        storage
            .set_last_seen_timestamps_private(ROOM, SELF, &[(BOB, unix_epoch(1000).into())])
            .await
            .unwrap();

        assert_eq!(
            storage
                .get_last_seen_timestamps_private(ROOM, SELF)
                .await
                .unwrap(),
            HashMap::from_iter([(BOB, unix_epoch(1000).into())])
        );

        storage
            .set_last_seen_timestamps_private(ROOM, SELF, &[(ALICE, unix_epoch(2000).into())])
            .await
            .unwrap();

        assert_eq!(
            storage
                .get_last_seen_timestamps_private(ROOM, SELF)
                .await
                .unwrap(),
            HashMap::from_iter([
                (BOB, unix_epoch(1000).into()),
                (ALICE, unix_epoch(2000).into()),
            ])
        );

        storage
            .delete_last_seen_timestamps_private(ROOM, SELF)
            .await
            .unwrap();

        assert!(storage
            .get_last_seen_timestamps_private(ROOM, SELF)
            .await
            .unwrap()
            .is_empty(),);
    }

    pub(super) async fn last_seen_private_is_personal(storage: &mut dyn ChatStorage) {
        // Set the private last seen timestamps as if BOB and ALICE were the participants in the
        // same room, and ensure this doesn't affect the timestamps of SELF.
        {
            // Set BOB's personal timestamps
            storage
                .set_last_seen_timestamps_private(
                    ROOM,
                    BOB,
                    &[
                        (ALICE, unix_epoch(1000).into()),
                        (SELF, unix_epoch(2000).into()),
                    ],
                )
                .await
                .unwrap();
        }
        {
            // Set ALICE's personal timestamps
            storage
                .set_last_seen_timestamps_private(ROOM, ALICE, &[(SELF, unix_epoch(3000).into())])
                .await
                .unwrap();
        }

        assert!(storage
            .get_last_seen_timestamps_private(ROOM, SELF)
            .await
            .unwrap()
            .is_empty());
    }
}
