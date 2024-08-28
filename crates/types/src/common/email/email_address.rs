// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Custom email address type wrapper. This is necessary because the `utoipa`
//! crate does not support the `email_address` crate.

#[allow(unused_imports)]
use crate::imports::*;
use crate::utils::ExampleData;

/// Representation of an email address
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    derive_more::FromStr,
    derive_more::AsRef,
    derive_more::Into,
    derive_more::Display,
)]
#[cfg_attr(feature = "serde", derive(Serialize, serde_with::DeserializeFromStr))]
pub struct EmailAddress(::email_address::EmailAddress);

impl EmailAddress {
    /// Extracts the string slace containing the entire `EmailAddress`
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Convert the mail address to lowercase
    pub fn to_lowercase(&self) -> Self {
        use std::str::FromStr;
        Self::from_str(self.as_str().to_lowercase().as_str()).unwrap()
    }
}

impl AsRef<str> for EmailAddress {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<EmailAddress> for String {
    fn from(value: EmailAddress) -> Self {
        Self::from(value.0)
    }
}

#[cfg(feature = "utoipa")]
impl<'__s> utoipa::ToSchema<'__s> for EmailAddress {
    fn schema() -> (
        &'__s str,
        utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
    ) {
        use serde_json::json;
        use utoipa::openapi::{SchemaFormat, SchemaType};

        (
            "EmailAddress",
            utoipa::openapi::ObjectBuilder::new()
                .schema_type(SchemaType::String)
                .format(Some(SchemaFormat::Custom("email".to_string())))
                .description(Some("An e-mail address"))
                .example(Some(json!(EmailAddress::example_data())))
                .into(),
        )
    }
}

impl ExampleData for EmailAddress {
    fn example_data() -> Self {
        "alice@example.com".parse().expect("email address")
    }
}
