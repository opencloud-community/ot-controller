// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 sip config endpoints.

use opentalk_types_common::{call_in::CallInPassword, utils::ExampleData};

#[allow(unused_imports)]
use crate::imports::*;

/// Body for the `PUT /rooms/{room_id}/sip` endpoint
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize, Validate),
    validate(schema(function = "disallow_empty"))
)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(PutSipConfig::example_data())),
)]
pub struct PutSipConfig {
    /// Numeric code required for entering the room. If not set explicitly on
    /// creation, this will be set to a randomly generated number.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub password: Option<CallInPassword>,

    /// Enable or disable the lobby for users that join throughh SIP. Defaults
    /// to [`false`] when not explicity set on creation.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub lobby: Option<bool>,
}

impl ExampleData for PutSipConfig {
    fn example_data() -> Self {
        Self {
            password: Some(CallInPassword::example_data()),
            lobby: Some(true),
        }
    }
}

#[cfg(feature = "serde")]
fn disallow_empty(modify_room: &PutSipConfig) -> Result<(), ValidationError> {
    let PutSipConfig { password, lobby } = modify_room;

    if password.is_none() && lobby.is_none() {
        Err(ValidationError::new("empty"))
    } else {
        Ok(())
    }
}
