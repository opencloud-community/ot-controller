// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Serialize, Serializer};

use crate::core::EventId;

use super::InstanceId;

/// Opaque id of an EventInstance or EventException resource. Should only be used to sort/index the related resource.
#[derive(Debug, Copy, Clone)]
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
