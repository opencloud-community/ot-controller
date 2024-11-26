// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_api_v1::rooms::RoomResource;

#[allow(unused_imports)]
use crate::imports::*;

/// The JSON body returned by the `/rooms` `GET` endpoint
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetRoomsResponse(pub Vec<RoomResource>);
