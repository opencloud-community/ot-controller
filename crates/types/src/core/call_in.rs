// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Display, From, FromStr, Into};

#[allow(unused_imports)]
use crate::imports::*;
use crate::utils::ExampleData;

/// The length of a numeric dial-in identifier
pub const DIAL_IN_NUMERIC_ID_LENGTH: usize = 10;

/// Base type for numeric dial-in identifieirs
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
    derive(ToRedisArgs, FromRedisValue,),
    to_redis_args(fmt),
    from_redis_value(FromStr)
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[cfg(feature = "serde")]
impl Validate for NumericId {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        if self.as_ref().len() != DIAL_IN_NUMERIC_ID_LENGTH {
            errors.add("0", ValidationError::new("Invalid id length"));
            return Err(errors);
        }

        for c in self.as_ref().chars() {
            if !c.is_ascii_digit() {
                errors.add("0", ValidationError::new("Non numeric character"));
                return Err(errors);
            }
        }

        Ok(())
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
    derive(ToRedisArgs, FromRedisValue,),
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

#[cfg(feature = "serde")]
impl Validate for CallInId {
    fn validate(&self) -> Result<(), ValidationErrors> {
        self.as_ref().validate()
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
    derive(ToRedisArgs, FromRedisValue,),
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

#[cfg(feature = "serde")]
impl Validate for CallInPassword {
    fn validate(&self) -> Result<(), ValidationErrors> {
        self.as_ref().validate()
    }
}
