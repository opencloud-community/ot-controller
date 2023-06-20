// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use uuid::Uuid;

crate::diesel_newtype! {
    feature_gated:

    #[derive(Copy)] InviteCodeId(uuid::Uuid) => diesel::sql_types::Uuid
}

impl InviteCodeId {
    /// Create a ZERO invite code id, e.g. for testing purposes
    pub const fn nil() -> Self {
        Self::from(Uuid::nil())
    }

    /// Create a invite code id from a number, e.g. for testing purposes
    pub const fn from_u128(id: u128) -> Self {
        Self::from(Uuid::from_u128(id))
    }

    /// Generate a new random invite code id
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        Self::from(Uuid::new_v4())
    }
}
