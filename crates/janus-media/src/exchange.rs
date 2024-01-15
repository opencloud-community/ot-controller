// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types::{
    core::ParticipantId,
    signaling::media::{command::ParticipantSelection, event::RequestMute},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    StartedTalking(ParticipantId),
    StoppedTalking(ParticipantId),
    RequestMute(RequestMute),
    PresenterGranted(ParticipantSelection),
    PresenterRevoked(ParticipantSelection),
}
