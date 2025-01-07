// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr;

use snafu::{ensure, Snafu};

use crate::utils::ExampleData;

/// The `core` module id.
pub const CORE_MODULE_ID: &str = "core";

/// The module that is used by default if none is specified
pub const DEFAULT_MODULE_ID: &str = CORE_MODULE_ID;

/// The minimum allowed length for a valid module id
pub const MIN_MODULE_ID_LENGTH: usize = 1;

/// The maximum allowed length for a valid module id
pub const MAX_MODULE_ID_LENGTH: usize = 255;

/// Regular expression of characters that are allowed inside a module id.
pub const MODULE_ID_SCHEMA_CHARS_REGEX: &str = "[-_0-9a-zA-Z]";

/// The id of a module.
///
/// Can be parsed using [`std::str::FromStr`].
/// May only contain alphanumeric ascii characters.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::Display)]
#[cfg_attr(
    feature = "diesel",
    derive(
        opentalk_diesel_newtype::DieselNewtype,
        diesel::expression::AsExpression,
        diesel::deserialize::FromSqlRow
    )
)]
#[cfg_attr(feature="diesel", diesel(sql_type = diesel::sql_types::Text))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde_with::DeserializeFromStr)
)]
pub struct ModuleId(String);

impl Default for ModuleId {
    fn default() -> Self {
        Self(DEFAULT_MODULE_ID.to_string())
    }
}

impl ModuleId {
    /// Check whether this module id is the default module id identified by [`DEFAULT_MODULE_ID`].
    pub fn is_default(&self) -> bool {
        self.0.as_str() == DEFAULT_MODULE_ID
    }
}

#[cfg(feature = "utoipa")]
mod impl_utoipa {
    //! The `#[derive(utoipa::ToSchema)] implementation does not yet properly support
    //! exposing schema information of types wrapped by the NewType pattern, therefore
    //! a manual implementation is required for now.
    //! Issue: <https://github.com/juhaku/utoipa/issues/663>

    use serde_json::json;
    use utoipa::{
        openapi::{schema::AnyOf, ObjectBuilder, RefOr, Schema, Type},
        PartialSchema, ToSchema,
    };

    use super::{
        ModuleId, MAX_MODULE_ID_LENGTH, MIN_MODULE_ID_LENGTH, MODULE_ID_SCHEMA_CHARS_REGEX,
    };
    use crate::utils::ExampleData as _;

    impl PartialSchema for ModuleId {
        fn schema() -> RefOr<Schema> {
            Schema::AnyOf(AnyOf {
                items: vec![ObjectBuilder::new()
                    .schema_type(Type::String)
                    .description(Some("A module identifier"))
                    .min_length(Some(MIN_MODULE_ID_LENGTH))
                    .max_length(Some(MAX_MODULE_ID_LENGTH))
                    .pattern(Some(format!("^{MODULE_ID_SCHEMA_CHARS_REGEX}*$")))
                    .examples([json!(ModuleId::example_data())])
                    .into()],
                description: None,
                default: Some(json!(ModuleId::default())),
                example: Some(json!(Self::example_data())),
                discriminator: None,
                ..Default::default()
            })
            .into()
        }
    }

    impl ToSchema for ModuleId {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}

impl ExampleData for ModuleId {
    fn example_data() -> Self {
        Self("mymodule".to_string())
    }
}

/// The error that is returned by [ModuleId::from_str] on failure.
#[derive(Debug, Snafu)]
pub enum ParseModuleIdError {
    /// Invalid characters were found in the input data.
    #[snafu(display("Module id may only contain alphanumeric characters, \"_\" or \"-\""))]
    InvalidCharacters,

    /// The input string was shorter than the minimum length [MIN_MODULE_ID_LENGTH].
    #[snafu(display("Module id must have at least {min_length} characters"))]
    TooShort {
        /// The minimum allowed length.
        min_length: usize,
    },

    /// The input string was longer than the maximum length [MAX_MODULE_ID_LENGTH].
    #[snafu(display("Module id must not be longer than {max_length} characters"))]
    TooLong {
        /// The maximum allowed length.
        max_length: usize,
    },
}

impl FromStr for ModuleId {
    type Err = ParseModuleIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure!(
            s.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
            InvalidCharactersSnafu
        );
        ensure!(
            s.len() >= MIN_MODULE_ID_LENGTH,
            TooShortSnafu {
                min_length: MIN_MODULE_ID_LENGTH
            }
        );
        ensure!(
            s.len() <= MAX_MODULE_ID_LENGTH,
            TooLongSnafu {
                max_length: MAX_MODULE_ID_LENGTH
            }
        );
        Ok(Self(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::{ModuleId, ParseModuleIdError};

    #[test]
    fn parse() {
        assert_eq!(
            "hello".parse::<ModuleId>().unwrap(),
            ModuleId("hello".to_string())
        );
        assert_eq!("_".parse::<ModuleId>().unwrap(), ModuleId("_".to_string()));
        assert_eq!(
            "hello_world".parse::<ModuleId>().unwrap(),
            ModuleId("hello_world".to_string())
        );
        assert_eq!("-".parse::<ModuleId>().unwrap(), ModuleId("-".to_string()));
        assert_eq!(
            "hello-world".parse::<ModuleId>().unwrap(),
            ModuleId("hello-world".to_string())
        );

        let longest: String = "x".repeat(255);
        assert_eq!(longest.parse::<ModuleId>().unwrap(), ModuleId(longest));
    }

    #[test]
    fn parse_invalid() {
        assert!(matches!(
            "".parse::<ModuleId>(),
            Err(ParseModuleIdError::TooShort { min_length: 1 })
        ));

        assert!(matches!(
            " ".parse::<ModuleId>(),
            Err(ParseModuleIdError::InvalidCharacters)
        ));

        assert!(matches!(
            "hello world".parse::<ModuleId>(),
            Err(ParseModuleIdError::InvalidCharacters)
        ));

        assert!(matches!(
            "hello+world".parse::<ModuleId>(),
            Err(ParseModuleIdError::InvalidCharacters)
        ));

        assert!(matches!(
            "hello~world".parse::<ModuleId>(),
            Err(ParseModuleIdError::InvalidCharacters)
        ));

        assert!(matches!(
            "hello::world".parse::<ModuleId>(),
            Err(ParseModuleIdError::InvalidCharacters)
        ));

        let too_long: String = "x".repeat(256);
        assert!(matches!(
            too_long.parse::<ModuleId>(),
            Err(ParseModuleIdError::TooLong { max_length: 255 })
        ));
    }
}
