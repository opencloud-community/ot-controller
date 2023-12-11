// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::core::RoomId;
#[allow(unused_imports)]
use crate::imports::*;

/// Response for the `POST /services/recording/upload_render` endpoint
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UploadRenderQuery {
    /// The room id
    pub room_id: RoomId,
    /// The filename
    pub filename: String,
}
