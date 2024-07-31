// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::core::{FileExtension, RoomId, Timestamp};
#[allow(unused_imports)]
use crate::imports::*;

/// Response for the `POST /services/recording/upload_render` endpoint
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::IntoParams))]
pub struct UploadRenderQuery {
    /// The room id
    pub room_id: RoomId,

    /// The file extension
    pub file_extension: FileExtension,

    /// The recording creation timestamp
    pub timestamp: Timestamp,
}
