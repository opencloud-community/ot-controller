// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::rooms::{invite_codes::InviteCode, RoomId};

/// Path for *GET /rooms/{room_id}/invites/{invite_code}*
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::IntoParams))]
pub struct RoomIdAndInviteCode {
    /// The room id for the invite
    pub room_id: RoomId,

    /// The invite code id
    pub invite_code: InviteCode,
}
