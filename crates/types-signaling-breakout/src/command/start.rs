// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use crate::command::RoomParameter;

/// Command to start a breakout session
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Start {
    /// A list of breakout rooms to create
    pub rooms: Vec<RoomParameter>,

    /// Duration of the breakout session
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            skip_serializing_if = "Option::is_none",
            with = "opentalk_types_common::utils::duration_seconds_option"
        )
    )]
    pub duration: Option<Duration>,
}
