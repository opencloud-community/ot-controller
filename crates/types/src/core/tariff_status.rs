// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::sql_enum;

#[allow(unused_imports)]
use crate::imports::*;

sql_enum!(
    feature_gated:

    #[derive(PartialEq, Eq)]
    #[cfg_attr(feature="serde", derive(Serialize, Deserialize), serde(rename_all = "snake_case"))]
    #[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
    TariffStatus,
    "tariff_status",
    TariffStatusType,
    {
        Default = b"default",
        Paid = b"paid",
        Downgraded = b"downgraded",
    }
);
