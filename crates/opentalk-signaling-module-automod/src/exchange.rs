// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Message types sent via the message exchange.
//!
//! Mostly duplicates of [`super::outgoing`] types.
//! See their respective originals for documentation.

use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_automod::{config::FrontendConfig, event::StoppedReason};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    Start(Start),
    Stop(StoppedReason),

    SpeakerUpdate(SpeakerUpdate),
    RemainingUpdate(RemainingUpdate),

    StartAnimation(StartAnimation),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Start {
    pub frontend_config: FrontendConfig,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SpeakerUpdate {
    pub speaker: Option<ParticipantId>,
    pub history: Option<Vec<ParticipantId>>,
    pub remaining: Option<Vec<ParticipantId>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RemainingUpdate {
    pub remaining: Vec<ParticipantId>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct StartAnimation {
    pub pool: Vec<ParticipantId>,
    pub result: ParticipantId,
}
