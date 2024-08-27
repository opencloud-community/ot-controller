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
    StreamingKind,
    "streaming_kind",
    StreamingKindType,
    {
        Custom = b"custom",
    }
);

impl Default for StreamingKind {
    fn default() -> Self {
        Self::Custom
    }
}
