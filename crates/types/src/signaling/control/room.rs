// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to the `JoinSuccess` message in the `control` namespace

use opentalk_types_common::rooms::{RoomId, RoomPassword};
use opentalk_types_signaling_control::room::CreatorInfo;

#[allow(unused_imports)]
use crate::imports::*;

/// Information about an room
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RoomInfo {
    /// The id of the room
    pub id: RoomId,

    /// The room password
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub password: Option<RoomPassword>,

    /// The room creator
    pub created_by: CreatorInfo,
}
