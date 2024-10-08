// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum Command {
    CreateNewAccessToken,
    ForceMute { participants: Vec<ParticipantId> },
    GrantScreenSharePermission { participants: Vec<ParticipantId> },
    RevokeScreenSharePermission { participants: Vec<ParticipantId> },
}
