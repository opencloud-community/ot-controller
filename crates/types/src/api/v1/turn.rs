// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 TURN endpoints.

#[allow(unused_imports)]
use crate::imports::*;

/// TURN access credentials for users.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Turn {
    /// The TURN access username
    pub username: String,

    /// The TURN access username
    pub password: String,

    /// Time to live of the TURN service
    pub ttl: String,

    /// URIs of the TURN service
    pub uris: Vec<String>,
}

/// STUN Server for users.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Stun {
    /// STUN URIs for this TURN server following rfc7065
    pub uris: Vec<String>,
}

/// Description of an ICE server
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(untagged))]
pub enum IceServer {
    /// TURN ICE server type
    Turn(Turn),

    /// STUN ICE server type
    Stun(Stun),
}

/// Response to the *GET /turn* endpoint request
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GetResponse(pub Vec<IceServer>);
