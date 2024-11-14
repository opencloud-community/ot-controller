// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::utils::ExampleData;

use super::InviteResource;
#[allow(unused_imports)]
use crate::imports::*;

/// Response for *GET /rooms/{room_id}/invites*
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature="utoipa",derive(utoipa::ToSchema), schema(example = json!(GetRoomsInvitesResponseBody::example_data())))]
pub struct GetRoomsInvitesResponseBody(pub Vec<InviteResource>);

impl ExampleData for GetRoomsInvitesResponseBody {
    fn example_data() -> Self {
        Self(vec![InviteResource::example_data()])
    }
}
