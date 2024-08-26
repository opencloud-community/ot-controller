// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr;

use opentalk_types_common::utils::ExampleData;
use snafu::{ensure, Snafu};

#[allow(unused_imports)]
use crate::imports::*;

/// The minimum allowed length for a valid room password
pub const MIN_ROOM_PASSWORD_LENGTH: usize = 1;

/// The maximum allowed length for a valid room password
pub const MAX_ROOM_PASSWORD_LENGTH: usize = 255;

/// A room password.
///
/// Can be parsed using [`std::str::FromStr`].
/// Must contain at least [`MIN_ROOM_PASSWORD_LENGTH`] characters, at most
/// [`MAX_ROOM_PASSWORD_LENGTH`] characters.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::Display)]
#[cfg_attr(
    feature = "diesel",
    derive(DieselNewtype, AsExpression, FromSqlRow),
    diesel(sql_type = diesel::sql_types::Text)
)]
#[cfg_attr(feature = "serde", derive(Serialize, serde_with::DeserializeFromStr))]
pub struct RoomPassword(String);

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

    use super::{RoomPassword, MAX_ROOM_PASSWORD_LENGTH, MIN_ROOM_PASSWORD_LENGTH};

    impl<'__s> ToSchema<'__s> for RoomPassword {
        fn schema() -> (
            &'__s str,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        ) {
            (
                "RoomPassword",
                ObjectBuilder::new()
                    .schema_type(SchemaType::String)
                    .description(Some("A room password"))
                    .min_length(Some(MIN_ROOM_PASSWORD_LENGTH))
                    .max_length(Some(MAX_ROOM_PASSWORD_LENGTH))
                    .example(Some("v3rys3cr3t".into()))
                    .into(),
            )
        }
    }
}

impl ExampleData for RoomPassword {
    fn example_data() -> Self {
        Self("v3rys3cr3t".to_string())
    }
}

impl std::fmt::Debug for RoomPassword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RoomPassword")
            .field(&format_args!("********"))
            .finish()
    }
}

#[derive(Debug, Snafu)]
pub enum ParseRoomPasswordError {
    #[snafu(display("Room password must have at least {min_length} characters"))]
    TooShort { min_length: usize },

    #[snafu(display("Room password must not be longer than {max_length} characters"))]
    TooLong { max_length: usize },
}

impl FromStr for RoomPassword {
    type Err = ParseRoomPasswordError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure!(
            s.len() >= MIN_ROOM_PASSWORD_LENGTH,
            TooShortSnafu {
                min_length: MIN_ROOM_PASSWORD_LENGTH
            }
        );
        ensure!(
            s.len() <= MAX_ROOM_PASSWORD_LENGTH,
            TooLongSnafu {
                max_length: MAX_ROOM_PASSWORD_LENGTH
            }
        );
        Ok(Self(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::{ParseRoomPasswordError, RoomPassword};

    #[test]
    fn parse() {
        assert_eq!(
            "hello".parse::<RoomPassword>().unwrap(),
            RoomPassword("hello".to_string())
        );
        assert_eq!(
            " ".parse::<RoomPassword>().unwrap(),
            RoomPassword(" ".to_string())
        );

        let longest: String = "x".repeat(255);
        assert_eq!(
            longest.parse::<RoomPassword>().unwrap(),
            RoomPassword(longest)
        );
    }

    #[test]
    fn parse_invalid() {
        assert!(matches!(
            "".parse::<RoomPassword>(),
            Err(ParseRoomPasswordError::TooShort { min_length: 1 })
        ));

        let too_long: String = "x".repeat(256);
        assert!(matches!(
            too_long.parse::<RoomPassword>(),
            Err(ParseRoomPasswordError::TooLong { max_length: 255 })
        ));
    }
}
