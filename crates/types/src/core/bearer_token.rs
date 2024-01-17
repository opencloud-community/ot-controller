// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{Display, From, FromStr, Into};

#[allow(unused_imports)]
use crate::imports::*;

/// A bearer token
#[derive(Display, From, FromStr, Into, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BearerToken(String);

impl BearerToken {
    /// Create a new bearer token
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into())
    }
}

#[cfg(feature = "frontend")]
impl opentalk_client_shared::Authorization for BearerToken {
    fn apply_authorization_headers(&self, headers: &mut http::HeaderMap) {
        let _ = headers.insert(
            http::header::AUTHORIZATION,
            http::HeaderValue::from_str(&format!("Bearer {}", self)).expect("valid header value"),
        );
    }
}
