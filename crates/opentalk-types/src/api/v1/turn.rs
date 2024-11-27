// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 TURN endpoints.

use opentalk_types_api_v1::turn::{StunServer, TurnServer};

#[allow(unused_imports)]
use crate::imports::*;

/// Description of an ICE server
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(untagged))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum IceServer {
    /// TURN ICE server type
    Turn(TurnServer),

    /// STUN ICE server type
    Stun(StunServer),
}

/// Response to the *GET /turn* endpoint request
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetTurnServersResponse(pub Vec<IceServer>);
