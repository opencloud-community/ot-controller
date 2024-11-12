// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling_subroom_audio::{
    event::{ParticipantsInvited, WhisperAccepted, WhisperInvite, WhisperParticipantInfo},
    whisper_id::WhisperId,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    WhisperInvite(WhisperInvite),
    ParticipantsInvited(ParticipantsInvited),
    WhisperAccepted(WhisperAccepted),
    WhisperDeclined(WhisperParticipantInfo),
    Kicked(WhisperId),
    LeftWhisperGroup(WhisperParticipantInfo),
}
