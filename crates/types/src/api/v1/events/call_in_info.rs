// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

/// Data stored inside the `GET /events` query cursor
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CallInInfo {
    /// SIP Call-In phone number which must be used to reach the room
    pub tel: String,

    /// SIP Call-In sip uri
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub uri: Option<String>,

    /// SIP ID which must transmitted via DTMF (number field on the phone) to identify this room
    pub id: String,

    /// SIP password which must be transmitted via DTMF (number field on the phone) after entering the `sip_id`
    /// to enter the room
    pub password: String,
}
