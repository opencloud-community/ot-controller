// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling_livekit::command::UnrestrictedParticipants;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    MicrophoneRestrictionsEnabled(UnrestrictedParticipants),
    MicrophoneRestrictionsDisabled,
}
