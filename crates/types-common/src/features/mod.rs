// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types for handling module features.

mod feature_id;
mod module_feature_id;

pub use feature_id::{
    FeatureId, ParseFeatureIdError, MAX_FEATURE_ID_LENGTH, MIN_FEATURE_ID_LENGTH,
};
pub use module_feature_id::{ModuleFeatureId, ParseModuleFeatureIdError};

/// The namespace separator
pub const NAMESPACE_SEPARATOR: &str = "::";
