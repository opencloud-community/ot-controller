// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_api_v1::assets::AssetResource;
use opentalk_types_common::utils::ExampleData;

#[allow(unused_imports)]
use crate::imports::*;

/// Response for *GET /rooms/{room_id}/assets*
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature="utoipa",derive(utoipa::ToSchema), schema(example = json!(GetRoomsAssetsResponseBody::example_data())))]
pub struct GetRoomsAssetsResponseBody(pub Vec<AssetResource>);

impl ExampleData for GetRoomsAssetsResponseBody {
    fn example_data() -> Self {
        Self(vec![AssetResource::example_data()])
    }
}
