// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr;

use snafu::{ensure, Snafu};

#[allow(unused_imports)]
use crate::imports::*;
use crate::utils::ExampleData;

/// The minimum allowed length for a valid feature id
pub const MIN_FEATURE_ID_LENGTH: usize = 1;

/// The maximum allowed length for a valid feature id
pub const MAX_FEATURE_ID_LENGTH: usize = 255;

/// The id of a feature.
///
/// Can be parsed using [`std::str::FromStr`].
/// May only contain alphanumeric ascii characters.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::Display)]
#[cfg_attr(
    feature = "diesel",
    derive(DieselNewtype, AsExpression, FromSqlRow),
    diesel(sql_type = diesel::sql_types::Text)
)]
#[cfg_attr(feature = "serde", derive(Serialize, serde_with::DeserializeFromStr))]
pub struct FeatureId(String);

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

    use super::{FeatureId, MAX_FEATURE_ID_LENGTH, MIN_FEATURE_ID_LENGTH};
    use crate::utils::ExampleData as _;

    impl<'__s> ToSchema<'__s> for FeatureId {
        fn schema() -> (
            &'__s str,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        ) {
            (
                "RoomPassword",
                ObjectBuilder::new()
                    .schema_type(SchemaType::String)
                    .description(Some("A feature identifier"))
                    .min_length(Some(MIN_FEATURE_ID_LENGTH))
                    .max_length(Some(MAX_FEATURE_ID_LENGTH))
                    .pattern(Some("^[-_0-9a-zA-Z]*$".to_string()))
                    .example(Some(FeatureId::example_data().to_string().into()))
                    .into(),
            )
        }
    }
}

impl ExampleData for FeatureId {
    fn example_data() -> Self {
        Self("myfeature".to_string())
    }
}

/// The error that is returned by [FeatureId::from_str] on failure.
#[derive(Debug, Snafu)]
pub enum ParseFeatureIdError {
    /// Invalid characters were found in the input data.
    #[snafu(display("Feature id may only contain alphanumeric characters, \"_\" or \"-\""))]
    InvalidCharacters,

    /// The input string was shorter than the minimum length [MIN_FEATURE_ID_LENGTH].
    #[snafu(display("Feature id must have at least {min_length} characters"))]
    TooShort {
        /// The minimum allowed length.
        min_length: usize,
    },

    /// The input string was longer than the maximum length [MAX_FEATURE_ID_LENGTH].
    #[snafu(display("Feature id must not be longer than {max_length} characters"))]
    TooLong {
        /// The maximum allowed length.
        max_length: usize,
    },
}

impl FromStr for FeatureId {
    type Err = ParseFeatureIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure!(
            s.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
            InvalidCharactersSnafu
        );
        ensure!(
            s.len() >= MIN_FEATURE_ID_LENGTH,
            TooShortSnafu {
                min_length: MIN_FEATURE_ID_LENGTH
            }
        );
        ensure!(
            s.len() <= MAX_FEATURE_ID_LENGTH,
            TooLongSnafu {
                max_length: MAX_FEATURE_ID_LENGTH
            }
        );
        Ok(Self(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::{FeatureId, ParseFeatureIdError};

    #[test]
    fn parse() {
        assert_eq!(
            "hello".parse::<FeatureId>().unwrap(),
            FeatureId("hello".to_string())
        );
        assert_eq!(
            "_".parse::<FeatureId>().unwrap(),
            FeatureId("_".to_string())
        );
        assert_eq!(
            "hello_world".parse::<FeatureId>().unwrap(),
            FeatureId("hello_world".to_string())
        );
        assert_eq!(
            "-".parse::<FeatureId>().unwrap(),
            FeatureId("-".to_string())
        );
        assert_eq!(
            "hello-world".parse::<FeatureId>().unwrap(),
            FeatureId("hello-world".to_string())
        );

        let longest: String = "x".repeat(255);
        assert_eq!(longest.parse::<FeatureId>().unwrap(), FeatureId(longest));
    }

    #[test]
    fn parse_invalid() {
        assert!(matches!(
            "".parse::<FeatureId>(),
            Err(ParseFeatureIdError::TooShort { min_length: 1 })
        ));

        assert!(matches!(
            " ".parse::<FeatureId>(),
            Err(ParseFeatureIdError::InvalidCharacters)
        ));

        assert!(matches!(
            "hello world".parse::<FeatureId>(),
            Err(ParseFeatureIdError::InvalidCharacters)
        ));

        assert!(matches!(
            "hello+world".parse::<FeatureId>(),
            Err(ParseFeatureIdError::InvalidCharacters)
        ));

        assert!(matches!(
            "hello~world".parse::<FeatureId>(),
            Err(ParseFeatureIdError::InvalidCharacters)
        ));

        assert!(matches!(
            "hello::world".parse::<FeatureId>(),
            Err(ParseFeatureIdError::InvalidCharacters)
        ));

        let too_long: String = "x".repeat(256);
        assert!(matches!(
            too_long.parse::<FeatureId>(),
            Err(ParseFeatureIdError::TooLong { max_length: 255 })
        ));
    }
}
