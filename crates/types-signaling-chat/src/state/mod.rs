// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling state for the `chat` namespace

mod group_history;
mod stored_message;

pub use group_history::GroupHistory;
pub use stored_message::StoredMessage;
