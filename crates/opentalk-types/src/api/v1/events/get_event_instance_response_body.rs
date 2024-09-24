// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_api_v1::events::EventInstance;
use opentalk_types_common::utils::ExampleData;

#[allow(unused_imports)]
use crate::imports::*;

/// Response for *GET /events/{event_id}/instances/{instance_id}*
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature="utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(GetEventInstancesResponseBody::example_data()))
)]
pub struct GetEventInstanceResponseBody(pub EventInstance);

impl ExampleData for GetEventInstanceResponseBody {
    fn example_data() -> Self {
        Self(EventInstance::example_data())
    }
}
