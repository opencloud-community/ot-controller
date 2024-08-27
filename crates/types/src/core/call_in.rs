// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Display, From, FromStr, Into};
use opentalk_types_common::utils::ExampleData;
use snafu::{ensure, Snafu};

#[allow(unused_imports)]
use crate::imports::*;

/// The length of a numeric dial-in identifier
pub const DIAL_IN_NUMERIC_ID_LENGTH: usize = 10;

/// Base type for numeric dial-in identifieirs
///
/// For now, this type checks via its `FromStr` implementation that the value is valid,
/// e.g. when deserializing through [`serde`](https://docs.rs/serde).
///
/// When loading from the database through [`diesel`](https://docs.rs/diesel), no validation
/// is performed in the current implementation.
#[derive(AsRef, Display, Into, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "diesel",
    derive(DieselNewtype, AsExpression, FromSqlRow),
    diesel(sql_type = diesel::sql_types::Text)
)]
#[cfg_attr(
    feature = "redis",
    derive(ToRedisArgs, FromRedisValue),
    to_redis_args(fmt),
    from_redis_value(FromStr)
)]
#[cfg_attr(feature = "serde", derive(Serialize, serde_with::DeserializeFromStr))]
pub struct NumericId(String);

impl NumericId {
    /// Generate a new random `NumericId`
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        use rand::{distributions::Slice, thread_rng, Rng as _};

        /// The set of numbers used to generate [`SipId`] & [`SipPassword`]
        const NUMERIC: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
        let numeric_dist = Slice::new(&NUMERIC).unwrap();

        Self(
            thread_rng()
                .sample_iter(numeric_dist)
                .take(DIAL_IN_NUMERIC_ID_LENGTH)
                .collect(),
        )
    }
}

#[derive(Debug, Snafu)]
pub enum ParseNumericIdError {
    #[snafu(display("Invalid numeric id length. Found: {found}, required: exactly {required}"))]
    InvalidLength { found: usize, required: usize },

    #[snafu(display("Invalid character found. Only numeric values are allowed"))]
    InvalidCharacter,
}

impl FromStr for NumericId {
    type Err = ParseNumericIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure!(
            s.len() == DIAL_IN_NUMERIC_ID_LENGTH,
            InvalidLengthSnafu {
                found: s.len(),
                required: DIAL_IN_NUMERIC_ID_LENGTH
            }
        );

        ensure!(s.chars().all(|c| c.is_ascii_digit()), InvalidCharacterSnafu);

        Ok(Self(s.to_string()))
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

    use super::{NumericId, DIAL_IN_NUMERIC_ID_LENGTH};

    impl<'__s> ToSchema<'__s> for NumericId {
        fn schema() -> (
            &'__s str,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        ) {
            (
                "NumericId",
                ObjectBuilder::new()
                    .schema_type(SchemaType::String)
                    .description(Some("A string containing number characters"))
                    .min_length(Some(DIAL_IN_NUMERIC_ID_LENGTH))
                    .max_length(Some(DIAL_IN_NUMERIC_ID_LENGTH))
                    .pattern(Some("[0-9]+"))
                    .example(Some("0000000000".into()))
                    .into(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NumericId, ParseNumericIdError, DIAL_IN_NUMERIC_ID_LENGTH};

    #[test]
    fn from_str() {
        assert!(matches!(
            "".parse::<NumericId>(),
            Err(ParseNumericIdError::InvalidLength {
                found: 0,
                required: DIAL_IN_NUMERIC_ID_LENGTH
            })
        ));

        assert!(matches!(
            "a".parse::<NumericId>(),
            Err(ParseNumericIdError::InvalidLength {
                found: 1,
                required: DIAL_IN_NUMERIC_ID_LENGTH
            })
        ));

        assert!(matches!(
            "123456789".parse::<NumericId>(),
            Err(ParseNumericIdError::InvalidLength {
                found: 9,
                required: DIAL_IN_NUMERIC_ID_LENGTH
            })
        ));

        assert!(matches!(
            "12345678900".parse::<NumericId>(),
            Err(ParseNumericIdError::InvalidLength {
                found: 11,
                required: DIAL_IN_NUMERIC_ID_LENGTH
            })
        ));

        assert!(matches!(
            "123456789a".parse::<NumericId>(),
            Err(ParseNumericIdError::InvalidCharacter)
        ));

        assert!(matches!(
            "abcdefghij".parse::<NumericId>(),
            Err(ParseNumericIdError::InvalidCharacter)
        ));

        assert!(matches!(
            "1234567890".parse::<NumericId>(),
            Ok(NumericId(s)) if s == "1234567890"
        ));

        assert!(matches!(
            "0000000000".parse::<NumericId>(),
            Ok(NumericId(s)) if s == "0000000000"
        ));

        assert!(matches!(
            "9999999999".parse::<NumericId>(),
            Ok(NumericId(s)) if s == "9999999999"
        ));
    }
}

/// The id of a call-in participation
#[derive(
    AsRef, Display, From, FromStr, Into, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[cfg_attr(
    feature = "diesel",
    derive(DieselNewtype, AsExpression, FromSqlRow),
    diesel(sql_type = diesel::sql_types::Text)
)]
#[cfg_attr(
    feature = "redis",
    derive(ToRedisArgs, FromRedisValue),
    to_redis_args(fmt),
    from_redis_value(FromStr)
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(CallInId::example_data())))]
pub struct CallInId(NumericId);

impl CallInId {
    /// Generate a random sip id
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        Self::from(NumericId::generate())
    }
}

impl ExampleData for CallInId {
    fn example_data() -> Self {
        Self(NumericId("0123456789".to_string()))
    }
}

/// The password for authenticating call-in participation
#[derive(
    AsRef, Display, From, FromStr, Into, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[cfg_attr(
    feature = "diesel",
    derive(DieselNewtype, AsExpression, FromSqlRow),
    diesel(sql_type = diesel::sql_types::Text)
)]
#[cfg_attr(
    feature = "redis",
    derive(ToRedisArgs, FromRedisValue),
    to_redis_args(fmt),
    from_redis_value(FromStr)
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(CallInPassword::example_data())))]
pub struct CallInPassword(NumericId);

impl CallInPassword {
    /// Generate a random sip password
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        Self::from(NumericId::generate())
    }
}

impl ExampleData for CallInPassword {
    fn example_data() -> Self {
        Self(NumericId("9876543210".to_string()))
    }
}
