// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::InstanceId;
#[allow(unused_imports)]
use crate::imports::*;
use crate::{core::EventId, utils::ExampleData};

/// Opaque id of an EventInstance or EventException resource. Should only be used to sort/index the related resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventAndInstanceId(pub EventId, pub InstanceId);

#[cfg(feature = "serde")]
mod serde_impls {
    use chrono::{DateTime, Utc};
    use serde::de::Error;

    use super::*;

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

            let instance_id: DateTime<Utc> = DateTime::parse_from_rfc3339(instance_id_str)
                .map_err(D::Error::custom)?
                .into();

            let event_id =
                EventId::from(uuid::Uuid::parse_str(event_id).map_err(D::Error::custom)?);

            Ok(EventAndInstanceId(event_id, instance_id.into()))
        }
    }
}

impl ExampleData for EventAndInstanceId {
    fn example_data() -> Self {
        Self(EventId::example_data(), InstanceId::example_data())
    }
}

#[cfg(feature = "utoipa")]
mod impl_to_schema {
    use serde_json::json;
    use utoipa::{
        openapi::{ObjectBuilder, SchemaType},
        ToSchema,
    };

    use super::EventAndInstanceId;
    use crate::utils::ExampleData as _;

    impl<'__s> ToSchema<'__s> for EventAndInstanceId {
        fn schema() -> (
            &'__s str,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        ) {
            (
                "EventAndInstanceId",
                ObjectBuilder::new()
                    .schema_type(SchemaType::String)
                    .description(Some("An event id and an instance id"))
                    .example(Some(json!(EventAndInstanceId::example_data())))
                    .into(),
            )
        }
    }
}
