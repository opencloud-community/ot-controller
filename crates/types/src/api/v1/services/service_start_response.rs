// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::auth::ResumptionToken;

use crate::core::TicketToken;
#[allow(unused_imports)]
use crate::imports::*;

/// Response for `POST /**/**/start` endpoints
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ServiceStartResponse {
    /// The ticket token
    pub ticket: TicketToken,
    /// The resumption token
    pub resumption: ResumptionToken,
}
