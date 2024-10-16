// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types for handling module features.

mod feature_id;
mod module_feature_id;

pub use feature_id::{
    FeatureId, ParseFeatureIdError, FEATURE_ID_SCHEMA_CHARS_REGEX, MAX_FEATURE_ID_LENGTH,
    MIN_FEATURE_ID_LENGTH,
};
pub use module_feature_id::{ModuleFeatureId, ParseModuleFeatureIdError};

use crate::modules::ModuleId;

/// The namespace separator
pub const NAMESPACE_SEPARATOR: &str = "::";

/// The call-in feature identifier string
pub const CALL_IN_FEATURE_ID: &str = "call_in";

/// The call-in module feature id
pub fn call_in() -> ModuleFeatureId {
    ModuleFeatureId {
        module: ModuleId::default(),
        feature: CALL_IN_FEATURE_ID.parse().expect("valid feature id"),
    }
}
