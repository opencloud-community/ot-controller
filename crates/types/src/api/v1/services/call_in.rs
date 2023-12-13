// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::core::{CallInId, CallInPassword};
#[allow(unused_imports)]
use crate::imports::*;

/// Body for the `POST /services/call_in/start` endpoint
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StartRequestBody {
    /// The call-in ID
    pub id: CallInId,
    /// The call-in password
    pub pin: CallInPassword,
}
