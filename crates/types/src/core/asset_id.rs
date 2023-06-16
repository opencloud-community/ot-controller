// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

crate::diesel_newtype! {
    feature_gated:

    #[derive(Copy)] AssetId(uuid::Uuid) => diesel::sql_types::Uuid, "diesel::sql_types::Uuid"
}

impl AssetId {
    /// Generate a new random participant id
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        Self::from(uuid::Uuid::new_v4())
    }
}
