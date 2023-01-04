// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Display, From, FromStr, Into};

#[allow(unused_imports)]
use crate::imports::*;

/// Base type for numeric identifieirs
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

        Self(thread_rng().sample_iter(numeric_dist).take(10).collect())
    }
}

#[cfg(feature = "serde")]
impl Validate for NumericId {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        if self.as_ref().len() != 10 {
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
pub struct CallInId(NumericId);

impl CallInId {
    /// Generate a random sip id
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        Self::from(NumericId::generate())
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
pub struct CallInPassword(NumericId);

impl CallInPassword {
    /// Generate a random sip password
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        Self::from(NumericId::generate())
    }
}

#[cfg(feature = "serde")]
impl Validate for CallInPassword {
    fn validate(&self) -> Result<(), ValidationErrors> {
        self.as_ref().validate()
    }
}
