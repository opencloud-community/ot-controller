// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Display, From, FromStr, Into};

use crate::utils::ExampleData;

/// Representation of a timezone
#[derive(AsRef, Display, From, FromStr, Into, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "diesel",  diesel(sql_type = diesel::sql_types::Text))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeZone(chrono_tz::Tz);

#[cfg(feature = "diesel")]
mod diesel_traits {
    use std::{
        io::Write,
        str::{from_utf8, FromStr},
    };

    use chrono_tz::Tz;
    use diesel::{
        backend::Backend,
        deserialize::{self, FromSql},
        pg::Pg,
        serialize::{self, IsNull, Output, ToSql},
    };

    use super::*;

    impl ToSql<diesel::sql_types::Text, Pg> for TimeZone {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
            write!(out, "{}", self.0)?;
            Ok(IsNull::No)
        }
    }

    impl FromSql<diesel::sql_types::Text, Pg> for TimeZone {
        fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
            let s = from_utf8(bytes.as_bytes())?;
            let tz = Tz::from_str(s)?;

            Ok(Self(tz))
        }
    }
}

impl ExampleData for TimeZone {
    fn example_data() -> Self {
        chrono_tz::Europe::Berlin.into()
    }
}

#[cfg(feature = "utoipa")]
mod impl_to_schema {
    use utoipa::{
        openapi::{ObjectBuilder, SchemaType},
        ToSchema,
    };

    use super::TimeZone;

    impl<'__s> ToSchema<'__s> for TimeZone {
        fn schema() -> (
            &'__s str,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        ) {
            (
                "TimeZone",
                ObjectBuilder::new()
                    .schema_type(SchemaType::String)
                    .description(Some("A time zone"))
                    .example(Some("Europe/Berlin".into()))
                    .into(),
            )
        }
    }
}
