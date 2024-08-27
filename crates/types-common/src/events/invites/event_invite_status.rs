// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::BTreeSet, str::FromStr};

#[allow(unused_imports)]
use crate::imports::*;
use crate::{sql_enum, utils::ExampleData};

sql_enum!(
    feature_gated:

    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[cfg_attr(feature="serde", derive(Serialize, Deserialize), serde(rename_all = "snake_case"))]
    #[cfg_attr(
        feature = "utoipa",
        derive(utoipa::ToSchema),
        schema(example = json!(EventInviteStatus::example_data()))
    )]
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

impl FromStr for EventInviteStatus {
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

impl EventInviteStatus {
    /// Get all values for this enumeration type
    pub fn all_enum_values() -> BTreeSet<Self> {
        BTreeSet::from_iter([
            Self::Pending,
            Self::Accepted,
            Self::Tentative,
            Self::Declined,
        ])
    }
}

impl ExampleData for EventInviteStatus {
    fn example_data() -> Self {
        Self::Accepted
    }
}
