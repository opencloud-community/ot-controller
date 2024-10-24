// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::Error;
use crate::state;

/// The events emitted for livekit
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LiveKitEvent {
    /// The credentials for a client to use livekit
    State(state::LiveKitState),
    /// An error e
    Error(Error),
}
