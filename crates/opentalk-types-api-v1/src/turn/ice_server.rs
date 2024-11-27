// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::{StunServer, TurnServer};

/// Description of an ICE server
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(untagged)
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum IceServer {
    /// TURN ICE server type
    Turn(TurnServer),

    /// STUN ICE server type
    Stun(StunServer),
}
