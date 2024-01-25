// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};

#[allow(unused_imports)]
use crate::imports::*;

/// Body for *POST /rooms/{room_id}/invites*
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PostInviteRequestBody {
    /// Optional expiration date of the invite
    pub expiration: Option<DateTime<Utc>>,
}
