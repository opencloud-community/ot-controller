// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_api_v1::events::InstanceId;
use opentalk_types_common::events::EventId;

#[allow(unused_imports)]
use crate::imports::*;

/// Path parameters for the `GET /events/{event_id}/instances/{instance_id}` endpoint
#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature="utoipa",
    derive(utoipa::IntoParams),
    into_params(parameter_in = Path),
)]
pub struct EventInstancePath {
    /// ID of the event
    pub event_id: EventId,

    /// ID of the event instance
    pub instance_id: InstanceId,
}
