// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use email_address::EmailAddress;

#[allow(unused_imports)]
use crate::imports::*;

/// Query parameters for the `DELETE /events/{event_id}/invites/email` endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DeleteEmailInviteBody {
    /// Email address of the user to delete the invite for
    pub email: EmailAddress,
}
