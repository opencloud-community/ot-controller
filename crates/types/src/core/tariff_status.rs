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
    TariffStatus,
    "tariff_status",
    TariffStatusType,
    {
        Default = b"default",
        Paid = b"paid",
        Downgraded = b"downgraded",
    }
);
