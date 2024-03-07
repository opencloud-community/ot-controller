// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Error response types for REST APIv1

#[cfg(any(feature = "backend", feature = "frontend"))]
mod api_error;
mod authentication_error;
mod error_body;
mod standard_error_body;
mod validation_error_body;
mod validation_error_entry;

#[cfg(any(feature = "backend", feature = "frontend"))]
pub use api_error::ApiError;
pub use authentication_error::AuthenticationError;
pub use error_body::ErrorBody;
pub use standard_error_body::StandardErrorBody;
pub use validation_error_body::ValidationErrorBody;
pub use validation_error_entry::ValidationErrorEntry;

/// Error code for an invalid URL
pub const ERROR_CODE_INVALID_URL: &str = "invalid_url";

/// Error code for an invalid E-Mail address
pub const ERROR_CODE_INVALID_EMAIL: &str = "invalid_email";

/// Error code for an invalid length
pub const ERROR_CODE_INVALID_LENGTH: &str = "invalid_length";

/// Error code for out of range access
pub const ERROR_CODE_OUT_OF_RANGE: &str = "out_of_range";

/// Error code when a required value is missing
pub const ERROR_CODE_VALUE_REQUIRED: &str = "value_required";

/// Error code when an existing value is ignored
pub const ERROR_CODE_IGNORED_VALUE: &str = "ignored_value";

/// Error code when required values are missing
pub const ERROR_CODE_MISSING_VALUE: &str = "missing_values";

/// Error code when an invalid value is encountered
pub const ERROR_CODE_INVALID_VALUE: &str = "invalid_value";
