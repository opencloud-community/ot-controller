// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr;

use snafu::{ensure, Snafu};

use crate::utils::ExampleData;

/// The maximum allowed number of characters for a [`Language`]
pub const MAX_LANGUAGE_LENGTH: usize = 35;

/// A language identifier
///
/// Can be parsed using [`std::str::FromStr`].
#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::Display)]
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
pub struct Language(String);

impl Language {
    /// Returns `true` if this `Language` has a length of zero, and `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Create a new empty [`Language`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the `&str` reference to the language string
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Get the length of the language string (in characters)
    pub fn len(&self) -> usize {
        self.0.chars().count()
    }
}

#[cfg(feature = "utoipa")]
mod impl_to_schema {
    //! The `#[derive(utoipa::ToSchema)] implementation does not yet properly support
    //! exposing schema information of types wrapped by the NewType pattern, therefore
    //! a manual implementation is required for now.
    //! Issue: <https://github.com/juhaku/utoipa/issues/663>

    use serde_json::json;
    use utoipa::{
        openapi::{ObjectBuilder, RefOr, Schema, Type},
        PartialSchema, ToSchema,
    };

    use super::{Language, MAX_LANGUAGE_LENGTH};
    use crate::utils::ExampleData as _;

    impl PartialSchema for Language {
        fn schema() -> RefOr<Schema> {
            ObjectBuilder::new()
                .schema_type(Type::String)
                .description(Some("A language identifier"))
                .max_length(Some(MAX_LANGUAGE_LENGTH))
                .examples([json!(Language::example_data())])
                .into()
        }
    }

    impl ToSchema for Language {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}

impl ExampleData for Language {
    fn example_data() -> Self {
        Self("de".to_string())
    }
}

/// The error that is returned by [Language::from_str] on failure.
#[derive(Debug, Snafu)]
pub enum ParseLanguageError {
    /// The input string was longer than the maximum length [MAX_LANGUAGE_LENGTH].
    #[snafu(display("Language must not be longer than {max_length} characters"))]
    TooLong {
        /// The maximum allowed length.
        max_length: usize,
    },
}

impl FromStr for Language {
    type Err = ParseLanguageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure!(
            s.len() <= MAX_LANGUAGE_LENGTH,
            TooLongSnafu {
                max_length: MAX_LANGUAGE_LENGTH
            }
        );
        Ok(Self(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::{Language, ParseLanguageError};

    #[test]
    fn parse() {
        assert_eq!(
            "hello".parse::<Language>().unwrap(),
            Language("hello".to_string())
        );
        assert_eq!("".parse::<Language>().unwrap(), Language("".to_string()));
        assert_eq!("_".parse::<Language>().unwrap(), Language("_".to_string()));
        assert_eq!(
            "hello world".parse::<Language>().unwrap(),
            Language("hello world".to_string())
        );
        assert_eq!(
            "🚀".parse::<Language>().unwrap(),
            Language("🚀".to_string())
        );

        let longest: String = "x".repeat(35);
        assert_eq!(longest.parse::<Language>().unwrap(), Language(longest));
    }

    #[test]
    fn parse_invalid() {
        let too_long: String = "x".repeat(36);
        assert!(matches!(
            too_long.parse::<Language>(),
            Err(ParseLanguageError::TooLong { max_length: 35 })
        ));
    }
}
