// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::core::EventId;
#[allow(unused_imports)]
use crate::imports::*;

use super::InstanceId;

/// Path parameters for the `GET /events/{event_id}/instances/{instance_id}` endpoint
#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EventInstancePath {
    /// ID of the event
    pub event_id: EventId,
    /// ID of the event instance
    pub instance_id: InstanceId,
}
