// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{Display, From, FromStr, Into};

#[allow(unused_imports)]
use crate::imports::*;

/// A ticket token
#[derive(Display, From, FromStr, Into, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TicketToken(pub String);

impl TicketToken {
    /// Get a str reference to the data in the ticket token
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
