// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod polls_storage;
mod redis;
mod volatile;

pub(crate) use polls_storage::PollsStorage;
// TODO: remove these re-exports once available in the PollsStorage trait
pub(crate) use redis::{
    del_results, del_state, list_add, list_members, poll_results, set_state, vote,
};
