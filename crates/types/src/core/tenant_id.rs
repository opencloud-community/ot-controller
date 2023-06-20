// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use uuid::Uuid;

crate::diesel_newtype! {
    feature_gated:

    #[derive(Copy)]
    TenantId(uuid::Uuid) => diesel::sql_types::Uuid
}

impl TenantId {
    /// Create a ZERO tenant id, e.g. for testing purposes
    pub const fn nil() -> Self {
        Self::from(Uuid::nil())
    }

    /// Create a tenant id from a number, e.g. for testing purposes
    pub const fn from_u128(id: u128) -> Self {
        Self::from(Uuid::from_u128(id))
    }

    /// Generate a new random tenant id
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        Self::from(Uuid::new_v4())
    }
}
