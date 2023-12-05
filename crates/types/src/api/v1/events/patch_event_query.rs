// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

/// Path query parameters for the `PATCH /events/{event_id}` endpoint
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PatchEventQuery {
    /// Maximum number of invitees to include inside the event
    #[cfg_attr(feature = "serde", serde(default))]
    pub invitees_max: i64,

    /// Flag to disable email notification
    #[cfg_attr(feature = "serde", serde(default))]
    pub suppress_email_notification: bool,
}
