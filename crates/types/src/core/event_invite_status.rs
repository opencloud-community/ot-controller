// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

use crate::sql_enum;

sql_enum!(
    feature_gated:

    #[derive(PartialEq, Eq)]
    #[cfg_attr(feature="serde", derive(Serialize, Deserialize), serde(rename_all = "snake_case"))]
    EventInviteStatus,
    "event_invite_status",
    EventInviteStatusType,
    {
        Pending = b"pending",
        Accepted = b"accepted",
        Tentative = b"tentative",
        Declined = b"declined",
    }
);

impl std::str::FromStr for EventInviteStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "accepted" => Ok(Self::Accepted),
            "tentative" => Ok(Self::Tentative),
            "declined" => Ok(Self::Declined),
            _ => Err(format!("unknown invite_status {s:?}")),
        }
    }
}
