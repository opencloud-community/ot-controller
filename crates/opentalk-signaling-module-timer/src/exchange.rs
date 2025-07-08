// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_timer::{
    TimerId,
    event::{Started, Stopped},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    Start(Started),
    Stop(Stopped),
    /// A participant updated its ready status
    UpdateReadyStatus(UpdateReadyStatus),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateReadyStatus {
    /// The timer that the update is for
    pub timer_id: TimerId,
    /// The participant that issued the update
    pub participant_id: ParticipantId,
}
