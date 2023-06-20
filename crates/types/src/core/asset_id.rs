// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use uuid::Uuid;

crate::diesel_newtype! {
    feature_gated:

    #[derive(Copy)] AssetId(uuid::Uuid) => diesel::sql_types::Uuid, "diesel::sql_types::Uuid"
}

impl AssetId {
    /// Create a ZERO asset id, e.g. for testing purposes
    pub const fn nil() -> Self {
        Self::from(Uuid::nil())
    }

    /// Create an asset id from a number, e.g. for testing purposes
    pub const fn from_u128(id: u128) -> Self {
        Self::from(Uuid::from_u128(id))
    }

    /// Generate a new random asset id
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        Self::from(Uuid::new_v4())
    }
}
