// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

/// Query parameters for the `PATCH /events/{event_id}/invites` endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PostEventInviteQuery {
    /// Flag to suppress email notification
    #[cfg_attr(feature = "serde", serde(default))]
    pub suppress_email_notification: bool,
}
