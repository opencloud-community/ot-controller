// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{auth::ResumptionToken, utils::ExampleData};

use crate::core::TicketToken;
#[allow(unused_imports)]
use crate::imports::*;

/// The JSON body returned from the start endpoints supporting session resumption
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(RoomsStartResponse::example_data())))]
pub struct RoomsStartResponse {
    /// The ticket token for the room
    pub ticket: TicketToken,

    /// The resumption token for the room
    pub resumption: ResumptionToken,
}

impl ExampleData for RoomsStartResponse {
    fn example_data() -> Self {
        Self {
            ticket: TicketToken::example_data(),
            resumption: ResumptionToken::example_data(),
        }
    }
}
