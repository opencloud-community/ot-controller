// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::InviteResource;
#[allow(unused_imports)]
use crate::imports::*;

/// Response for *GET /rooms/{room_id}/invites*
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GetRoomsInvitesResponseBody(pub Vec<InviteResource>);
