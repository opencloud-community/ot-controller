// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr;

use snafu::{ensure, Snafu};

use crate::utils::ExampleData;

pub const MAX_EVENT_DESCRIPTION_LENGTH: usize = 4096;

/// The description of an event.
///
/// Can be parsed using [`std::str::FromStr`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::Display)]
#[cfg_attr(
    feature = "diesel",
    derive(
        opentalk_diesel_newtype::DieselNewtype,
        diesel::expression::AsExpression,
        diesel::deserialize::FromSqlRow
    )
)]
#[cfg_attr(
    feature = "diesel",
    diesel(sql_type = diesel::sql_types::Text)
)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde_with::DeserializeFromStr)
)]
pub struct EventDescription(String);

impl EventDescription {
    /// Returns `true` if this `EventDescription` has a length of zero, and `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(feature = "utoipa")]
mod impl_to_schema {
    //! The `#[derive(utoipa::ToSchema)] implementation does not yet properly support
    //! exposing schema information of types wrapped by the NewType pattern, therefore
    //! a manual implementation is required for now.
    //! Issue: <https://github.com/juhaku/utoipa/issues/663>

    use utoipa::{
        openapi::{ObjectBuilder, SchemaType},
        ToSchema,
    };

    use super::{EventDescription, MAX_EVENT_DESCRIPTION_LENGTH};
    use crate::utils::ExampleData as _;

    impl<'__s> ToSchema<'__s> for EventDescription {
        fn schema() -> (
            &'__s str,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        ) {
            (
                "EventDescription",
                ObjectBuilder::new()
                    .schema_type(SchemaType::String)
                    .description(Some("The description of an event"))
                    .max_length(Some(MAX_EVENT_DESCRIPTION_LENGTH))
                    .example(Some(EventDescription::example_data().to_string().into()))
                    .into(),
            )
        }
    }
}

impl ExampleData for EventDescription {
    fn example_data() -> Self {
        Self("The Weekly Team Event".to_string())
    }
}

/// The error that is returned by [ModuleId::from_str] on failure.
#[derive(Debug, Snafu)]
pub enum ParseEventDescriptionError {
    /// The input string was longer than the maximum length [MAX_EVENT_DESCRIPTION_LENGTH].
    #[snafu(display("Event description must not be longer than {max_length} characters"))]
    TooLong {
        /// The maximum allowed length.
        max_length: usize,
    },
}

impl FromStr for EventDescription {
    type Err = ParseEventDescriptionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure!(
            s.len() <= MAX_EVENT_DESCRIPTION_LENGTH,
            TooLongSnafu {
                max_length: MAX_EVENT_DESCRIPTION_LENGTH
            }
        );
        Ok(Self(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::{EventDescription, ParseEventDescriptionError};

    #[test]
    fn parse() {
        assert_eq!(
            "hello".parse::<EventDescription>().unwrap(),
            EventDescription("hello".to_string())
        );
        assert_eq!(
            "".parse::<EventDescription>().unwrap(),
            EventDescription("".to_string())
        );
        assert_eq!(
            "_".parse::<EventDescription>().unwrap(),
            EventDescription("_".to_string())
        );
        assert_eq!(
            "hello world".parse::<EventDescription>().unwrap(),
            EventDescription("hello world".to_string())
        );
        assert_eq!(
            "🚀".parse::<EventDescription>().unwrap(),
            EventDescription("🚀".to_string())
        );

        let longest: String = "x".repeat(4096);
        assert_eq!(
            longest.parse::<EventDescription>().unwrap(),
            EventDescription(longest)
        );
    }

    #[test]
    fn parse_invalid() {
        let too_long: String = "x".repeat(4097);
        assert!(matches!(
            too_long.parse::<EventDescription>(),
            Err(ParseEventDescriptionError::TooLong { max_length: 4096 })
        ));
    }
}
