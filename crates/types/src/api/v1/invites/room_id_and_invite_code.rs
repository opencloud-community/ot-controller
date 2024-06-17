// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::core::{InviteCodeId, RoomId};
#[allow(unused_imports)]
use crate::imports::*;

/// Path for *GET /rooms/{room_id}/invites/{invite_code}*
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::IntoParams))]
pub struct RoomIdAndInviteCode {
    /// The room id for the invite
    pub room_id: RoomId,

    /// The invite code id
    pub invite_code: InviteCodeId,
}
