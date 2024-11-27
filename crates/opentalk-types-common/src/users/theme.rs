// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr;

use snafu::{ensure, Snafu};

use crate::utils::ExampleData;

/// The maximum allowed number of characters for a [`Theme`]
pub const MAX_THEME_LENGTH: usize = 128;

/// A theme identifier
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
pub struct Theme(String);

impl Theme {
    /// Returns `true` if this `Theme` has a length of zero, and `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Create a new empty [`Theme`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the `&str` reference to the theme identifier
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Get the length of the theme identifier (in characters)
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

    use utoipa::{
        openapi::{ObjectBuilder, SchemaType},
        ToSchema,
    };

    use super::{Theme, MAX_THEME_LENGTH};
    use crate::utils::ExampleData as _;

    impl<'__s> ToSchema<'__s> for Theme {
        fn schema() -> (
            &'__s str,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        ) {
            (
                "Theme",
                ObjectBuilder::new()
                    .schema_type(SchemaType::String)
                    .description(Some("A theme identifier"))
                    .max_length(Some(MAX_THEME_LENGTH))
                    .example(Some(Theme::example_data().to_string().into()))
                    .into(),
            )
        }
    }
}

impl ExampleData for Theme {
    fn example_data() -> Self {
        Self("de".to_string())
    }
}

/// The error that is returned by [Theme::from_str] on failure.
#[derive(Debug, Snafu)]
pub enum ParseThemeError {
    /// The input string was longer than the maximum length [MAX_THEME_LENGTH].
    #[snafu(display("Theme must not be longer than {max_length} characters"))]
    TooLong {
        /// The maximum allowed length.
        max_length: usize,
    },
}

impl FromStr for Theme {
    type Err = ParseThemeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure!(
            s.len() <= MAX_THEME_LENGTH,
            TooLongSnafu {
                max_length: MAX_THEME_LENGTH
            }
        );
        Ok(Self(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::{ParseThemeError, Theme};

    #[test]
    fn parse() {
        assert_eq!(
            "hello".parse::<Theme>().unwrap(),
            Theme("hello".to_string())
        );
        assert_eq!("".parse::<Theme>().unwrap(), Theme("".to_string()));
        assert_eq!("_".parse::<Theme>().unwrap(), Theme("_".to_string()));
        assert_eq!(
            "hello world".parse::<Theme>().unwrap(),
            Theme("hello world".to_string())
        );
        assert_eq!("🚀".parse::<Theme>().unwrap(), Theme("🚀".to_string()));

        let longest: String = "x".repeat(128);
        assert_eq!(longest.parse::<Theme>().unwrap(), Theme(longest));
    }

    #[test]
    fn parse_invalid() {
        let too_long: String = "x".repeat(129);
        assert!(matches!(
            too_long.parse::<Theme>(),
            Err(ParseThemeError::TooLong { max_length: 128 })
        ));
    }
}
