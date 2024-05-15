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
    add_participant_to_set, decrement_participant_count, delete_event, delete_participant_count,
    delete_tariff, get_attribute, get_attribute_for_participants, get_event, get_participant_count,
    get_role_and_left_at_for_room_participants, get_room_closes_at, get_skip_waiting_room,
    get_tariff, increment_participant_count, participant_id_in_use, participants_all_left,
    participants_contains, remove_attribute, remove_attribute_key, remove_room_closes_at,
    reset_skip_waiting_room_expiry, room_mutex, set_attribute, set_room_closes_at,
    set_skip_waiting_room_with_expiry, set_skip_waiting_room_with_expiry_nx, try_init_event,
    try_init_tariff, AttrPipeline, ParticipantIdRunnerLock,
};
