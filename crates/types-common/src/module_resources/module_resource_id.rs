// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Display, From, FromStr, Into};
use uuid::Uuid;

#[allow(unused_imports)]
use crate::imports::*;

/// The id of a module resource
#[derive(
    AsRef, Display, From, FromStr, Into, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[cfg_attr(feature="diesel",
    derive(DieselNewtype, AsExpression, FromSqlRow),
    diesel(sql_type = diesel::sql_types::Uuid),
)]
#[cfg_attr(
    feature = "kustos",
    derive(KustosPrefix),
    kustos_prefix("/module_resources/")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ModuleResourceId(Uuid);

impl ModuleResourceId {
    /// Create a ZERO module resource id, e.g. for testing purposes
    pub const fn nil() -> Self {
        Self(Uuid::nil())
    }

    /// Create a module resource id from a number, e.g. for testing purposes
    pub const fn from_u128(id: u128) -> Self {
        Self(Uuid::from_u128(id))
    }

    /// Generate a new random module resource id
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}
