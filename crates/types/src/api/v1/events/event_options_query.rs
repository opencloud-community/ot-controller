// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

/// Query parameters for miscellaneous /events and /events/{event_id}/...` endpoints
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EventOptionsQuery {
    /// Flag to disable email notification
    #[cfg_attr(feature = "serde", serde(default))]
    pub suppress_email_notification: bool,
}
