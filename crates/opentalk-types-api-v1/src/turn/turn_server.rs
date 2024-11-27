// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// TURN access credentials for users.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TurnServer {
    /// The TURN access username
    pub username: String,

    /// The TURN access username
    pub password: String,

    /// Time to live of the TURN service
    pub ttl: String,

    /// URIs of the TURN service
    pub uris: Vec<String>,
}
