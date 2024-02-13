// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::borrow::Cow;

use super::ValidationErrorEntry;
#[allow(unused_imports)]
use crate::imports::*;

/// The body of a validation error response
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ValidationErrorBody {
    /// Machine readable error message
    pub code: Cow<'static, str>,

    /// Human readable message
    pub message: Cow<'static, str>,

    /// A list validation errors
    pub errors: Vec<ValidationErrorEntry>,
}

impl ValidationErrorBody {
    /// Create a new validation error body with a message and a list of error entries
    pub fn new<C, M>(code: C, message: M, errors: Vec<ValidationErrorEntry>) -> Self
    where
        C: Into<Cow<'static, str>>,
        M: Into<Cow<'static, str>>,
    {
        Self {
            code: code.into(),
            message: message.into(),
            errors,
        }
    }
}
