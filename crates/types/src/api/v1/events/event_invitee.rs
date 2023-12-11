// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::core::EventInviteStatus;
#[allow(unused_imports)]
use crate::imports::*;

use super::EventInviteeProfile;

/// Invitee to an event
///
///  Contains user profile and invitee status
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EventInvitee {
    /// User profile of the invitee
    pub profile: EventInviteeProfile,
    /// Invite status of the invitee
    pub status: EventInviteStatus,
}
