// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::core::{ResumptionToken, TicketToken};

#[allow(unused_imports)]
use crate::imports::*;

/// The JSON body returned from the start endpoints supporting session resumption
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RoomsStartResponse {
    /// The ticket token for the room
    pub ticket: TicketToken,

    /// The resumption token for the room
    pub resumption: ResumptionToken,
}
