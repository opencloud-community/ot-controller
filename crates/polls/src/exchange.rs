// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};
use types::signaling::polls::{state::PollsState, PollId};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Started(PollsState),
    Update(PollId),
    Finish(PollId),
}
