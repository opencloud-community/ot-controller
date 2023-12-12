// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

use crate::core::EventId;

use super::InstanceId;

/// Opaque id of an EventInstance or EventException resource. Should only be used to sort/index the related resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventAndInstanceId(pub EventId, pub InstanceId);

#[cfg(feature = "serde")]
mod serde_impls {
    use super::*;
    use chrono::{DateTime, Utc};
    use serde::de::Error;

    impl Serialize for EventAndInstanceId {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            format!("{}_{}", self.0, self.1).serialize(serializer)
        }
    }

    #[cfg(feature = "serde")]
    impl<'de> Deserialize<'de> for EventAndInstanceId {
        fn deserialize<D>(deserializer: D) -> Result<EventAndInstanceId, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            let mut split = s.split('_');
            let event_id = split
                .next()
                .ok_or_else(|| D::Error::custom("missing event id"))?;
            let instance_id_str = split
                .next()
                .ok_or_else(|| D::Error::custom("missing instance id"))?;
            if split.next().is_some() {
                return Err(D::Error::custom("too many parts"));
            }

            let instance_id: DateTime<Utc> = chrono::DateTime::parse_from_rfc3339(instance_id_str)
                .map_err(D::Error::custom)?
                .into();

            let event_id =
                EventId::from(uuid::Uuid::parse_str(event_id).map_err(D::Error::custom)?);

            Ok(EventAndInstanceId(event_id, instance_id.into()))
        }
    }
}
