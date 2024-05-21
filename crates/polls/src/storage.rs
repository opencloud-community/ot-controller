// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod redis;

// TODO: remove these re-exports once available in the PollsStorage trait
pub(crate) use redis::{
    del_results, del_state, get_state, list_add, list_members, poll_results, set_state, vote,
};
