// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::{CallInId, CallInPassword};
#[allow(unused_imports)]
use crate::imports::*;
use crate::utils::ExampleData;

/// Information needed to participate in a call-in connection.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(CallInInfo::example_data()))
)]
pub struct CallInInfo {
    /// SIP Call-In phone number which must be used to reach the room
    pub tel: String,

    /// SIP ID which must transmitted via DTMF (number field on the phone) to identify this room
    pub id: CallInId,

    /// SIP password which must be transmitted via DTMF (number field on the phone) after entering the `sip_id`
    /// to enter the room
    pub password: CallInPassword,
}

impl ExampleData for CallInInfo {
    fn example_data() -> Self {
        Self {
            tel: "+555-123-456-789".to_string(),
            id: "1234567890".parse().expect("valid call-in id"),
            password: "0987654321".parse().expect("valid call-in password"),
        }
    }
}
