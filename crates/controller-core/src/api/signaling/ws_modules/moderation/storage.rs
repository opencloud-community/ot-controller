// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod redis;

// TODO: remove once everything is exposed through the ModerationStorage trait.
pub(crate) use redis::{
    ban_user, delete_bans, delete_raise_hands_enabled, delete_waiting_room,
    delete_waiting_room_accepted, delete_waiting_room_enabled, init_waiting_room_key, is_banned,
    is_raise_hands_enabled, is_waiting_room_enabled, set_raise_hands_enabled,
    set_waiting_room_enabled, waiting_room_accepted_add, waiting_room_accepted_all,
    waiting_room_accepted_len, waiting_room_accepted_remove, waiting_room_accepted_remove_list,
    waiting_room_add, waiting_room_all, waiting_room_contains, waiting_room_len,
    waiting_room_remove,
};
