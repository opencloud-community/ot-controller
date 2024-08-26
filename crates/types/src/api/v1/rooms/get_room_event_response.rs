// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::utils::ExampleData;

use crate::common::event::EventInfo;
#[allow(unused_imports)]
use crate::imports::*;

/// The JSON body returned by the `/rooms/<room_id>/event` endpoint
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetRoomEventResponse(pub EventInfo);

impl ExampleData for GetRoomEventResponse {
    fn example_data() -> Self {
        Self(EventInfo::example_data())
    }
}
