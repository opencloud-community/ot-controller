// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Error response types for REST APIv1

mod standard_error_body;
mod validation_error_body;
mod validation_error_entry;

pub use standard_error_body::StandardErrorBody;
pub use validation_error_body::ValidationErrorBody;
pub use validation_error_entry::ValidationErrorEntry;
