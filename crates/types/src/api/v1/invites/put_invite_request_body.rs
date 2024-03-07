// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};

#[allow(unused_imports)]
use crate::imports::*;

/// Body for *PUT /rooms/{room_id}/invites/{invite_code}*
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PutInviteRequestBody {
    /// Optional expiration date of the invite
    pub expiration: Option<DateTime<Utc>>,
}
