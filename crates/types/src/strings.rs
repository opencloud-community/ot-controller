// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains helpers for string-like types.

use email_address::EmailAddress;

/// Trait providing a function to convert string-like types to lowercase
pub trait ToLowerCase {
    /// Convert the type to lowercase
    fn to_lowercase(&self) -> Self;
}

impl ToLowerCase for EmailAddress {
    fn to_lowercase(&self) -> Self {
        use std::str::FromStr;
        Self::from_str(self.as_str().to_lowercase().as_str()).unwrap()
    }
}
