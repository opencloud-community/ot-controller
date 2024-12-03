// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{fmt::Display, ops::Add};

use chrono::{DateTime, TimeZone as _, Utc};
use opentalk_types_common::{time::Timestamp, utils::ExampleData};

use crate::events::UTC_DT_FORMAT;

/// ID of an EventInstance
///
/// Is created from the starts_at datetime of the original recurrence (original meaning that exceptions don't change
/// the instance id).
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, derive_more::From, derive_more::Into, derive_more::AsRef,
)]
pub struct InstanceId(Timestamp);

impl From<DateTime<Utc>> for InstanceId {
    fn from(dt: DateTime<Utc>) -> Self {
        InstanceId(dt.into())
    }
}

impl From<InstanceId> for DateTime<Utc> {
    fn from(id: InstanceId) -> Self {
        id.0.into()
    }
}

impl Display for InstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.format(UTC_DT_FORMAT))
    }
}

impl Add<chrono::Duration> for InstanceId {
    type Output = Self;

    fn add(self, rhs: chrono::Duration) -> Self::Output {
        InstanceId(self.0 + rhs)
    }
}

impl ExampleData for InstanceId {
    fn example_data() -> Self {
        Self(Utc.with_ymd_and_hms(2024, 7, 5, 17, 2, 42).unwrap().into())
    }
}

#[cfg(feature = "utoipa")]
mod impl_to_schema {
    use utoipa::{
        openapi::{ObjectBuilder, SchemaType},
        ToSchema,
    };

    use super::InstanceId;

    impl<'__s> ToSchema<'__s> for InstanceId {
        fn schema() -> (
            &'__s str,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        ) {
            (
                "InstanceId",
                ObjectBuilder::new()
                    .schema_type(SchemaType::String)
                    .description(Some("An event instance id"))
                    .example(Some("2024-07-20T15:23:42+00:00".into()))
                    .into(),
            )
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impls {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::{InstanceId, UTC_DT_FORMAT};

    const DT_FORMAT: &str = "%Y%m%dT%H%M%S%z";
    struct InstanceIdVisitor;

    impl serde::de::Visitor<'_> for InstanceIdVisitor {
        type Value = InstanceId;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(
                formatter,
                "timestamp in '{DT_FORMAT}' or '{UTC_DT_FORMAT}' format"
            )
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let utc_date_time = DateTime::parse_from_str(v, DT_FORMAT)
                .map(|dt| dt.with_timezone(&Utc))
                .or_else(|_| {
                    NaiveDateTime::parse_from_str(v, UTC_DT_FORMAT).map(|ndt| ndt.and_utc())
                })
                .map_err(|_| {
                    serde::de::Error::invalid_value(serde::de::Unexpected::Str(v), &self)
                })?;

            Ok(InstanceId(utc_date_time.into()))
        }
    }

    impl<'de> Deserialize<'de> for InstanceId {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_str(InstanceIdVisitor)
        }
    }

    impl Serialize for InstanceId {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.0
                .format(UTC_DT_FORMAT)
                .to_string()
                .serialize(serializer)
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use std::time::UNIX_EPOCH;

    use serde_json::Value;

    use super::InstanceId;

    #[test]
    fn serialize_utc() {
        let input = "19700101T000000Z";

        let instance_id: InstanceId = serde_json::from_value(Value::String(input.into())).unwrap();

        assert_eq!(instance_id.0, UNIX_EPOCH.into())
    }

    #[test]
    fn serialize_utc_plus_one() {
        let input = "19700101T010000+0100";

        let instance_id: InstanceId = serde_json::from_value(Value::String(input.into())).unwrap();

        assert_eq!(instance_id.0, UNIX_EPOCH.into())
    }
}
