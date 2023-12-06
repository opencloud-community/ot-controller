// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

use crate::core::EventId;

use super::InstanceId;

/// Opaque id of an EventInstance or EventException resource. Should only be used to sort/index the related resource.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EventAndInstanceId(pub EventId, pub InstanceId);

#[cfg(feature = "serde")]
impl Serialize for EventAndInstanceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        format!("{}_{}", self.0, self.1).serialize(serializer)
    }
}
