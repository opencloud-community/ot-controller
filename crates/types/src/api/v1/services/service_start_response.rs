// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::core::{ResumptionToken, TicketToken};
#[allow(unused_imports)]
use crate::imports::*;

/// Response for `POST /**/**/start` endpoints
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ServiceStartResponse {
    /// The ticket token
    pub ticket: TicketToken,
    /// The resumption token
    pub resumption: ResumptionToken,
}
