// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

use crate::sql_enum;

sql_enum!(
    feature_gated:

    #[derive(PartialEq, Eq, strum::Display)]
    #[cfg_attr(feature="serde", derive(Serialize, Deserialize), serde(rename_all = "snake_case"))]
    #[strum(serialize_all = "snake_case")]
    StreamingKind,
    "streaming_kind",
    StreamingKindType,
    {
        Builtin = b"builtin",
        Custom = b"custom",
        Provider = b"provider",
    }
);

impl Default for StreamingKind {
    fn default() -> Self {
        Self::Custom
    }
}
