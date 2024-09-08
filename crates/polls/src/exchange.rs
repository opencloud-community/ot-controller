// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types::signaling::polls::state::PollsState;
use opentalk_types_signaling_polls::PollId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Started(PollsState),
    Update(PollId),
    Finish(PollId),
}
