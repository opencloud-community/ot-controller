// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use serde::Deserialize;

use super::TurnServer;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct Turn {
    /// How long a credential pair be should be valid, in seconds
    #[serde(
        default,
        with = "opentalk_types_common::utils::duration_seconds_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub lifetime: Option<Duration>,

    /// List of configured TURN servers.
    pub servers: Option<Vec<TurnServer>>,
}
