// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! A list of known module and feature strings used by the `disabled_features` field in the controller settings.

use opentalk_types_common::features::ModuleFeatureId;

/// The call-in feature identifier string
pub const CALL_IN: &str = "core::call_in";

/// The call-in module feature id
pub fn call_in() -> ModuleFeatureId {
    CALL_IN.parse().expect("valid module feature id")
}
