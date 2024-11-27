// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 streaming target endpoints.

use opentalk_types_common::{rooms::RoomId, streaming::StreamingTargetId};

#[allow(unused_imports)]
use crate::imports::*;

/// The parameter set for /rooms/{room_id}/streaming_targets/{streaming_target_id}* endpoints
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::IntoParams))]
pub struct RoomAndStreamingTargetId {
    /// The room id for the invite
    pub room_id: RoomId,

    /// The streaming target id
    pub streaming_target_id: StreamingTargetId,
}
