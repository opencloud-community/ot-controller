// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::{StandardErrorBody, ValidationErrorBody};

/// The body of an error response
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(untagged)
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum ErrorBody {
    /// The standard error body
    Standard(StandardErrorBody),
    /// Special error body for validation errors
    Validation(ValidationErrorBody),
}
