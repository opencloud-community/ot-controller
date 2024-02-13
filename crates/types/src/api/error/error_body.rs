// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::{StandardErrorBody, ValidationErrorBody};
#[allow(unused_imports)]
use crate::imports::*;

/// The body of an error response
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(untagged))]
pub enum ErrorBody {
    /// The standard error body
    Standard(StandardErrorBody),
    /// Special error body for validation errors
    Validation(ValidationErrorBody),
}

impl ErrorBody {
    /// Get the content type for the corresponding body
    #[cfg(feature = "backend")]
    pub const fn content_type(&self) -> http::HeaderValue {
        http::HeaderValue::from_static("text/json; charset=utf-8")
    }
}
