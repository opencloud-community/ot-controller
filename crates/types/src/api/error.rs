// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Error response types for REST APIv1

mod error_body;
mod standard_error_body;
mod validation_error_body;
mod validation_error_entry;

pub use error_body::ErrorBody;
pub use standard_error_body::StandardErrorBody;
pub use validation_error_body::ValidationErrorBody;
pub use validation_error_entry::ValidationErrorEntry;
