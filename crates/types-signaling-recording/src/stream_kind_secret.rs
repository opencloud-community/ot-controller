// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::streaming::StreamingTargetKind;

/// The kind of the stream
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "streaming_kind", rename_all = "snake_case")
)]
pub enum StreamKindSecret {
    /// Recording kind
    Recording,
    /// Livestream kind
    Livestream(StreamingTargetKind),
}
