// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::streaming::StreamingTargetKind;
use url::Url;

use crate::StreamKindSecret;

/// The kind of the stream
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "streaming_kind", rename_all = "snake_case")
)]
pub enum StreamKind {
    /// Recording kind
    Recording,
    /// Livestream kind
    Livestream {
        /// The public url to the stream
        public_url: Url,
    },
}

impl From<StreamKindSecret> for StreamKind {
    fn from(val: StreamKindSecret) -> StreamKind {
        match val {
            StreamKindSecret::Recording => StreamKind::Recording,
            StreamKindSecret::Livestream(stk) => match stk {
                StreamingTargetKind::Custom {
                    streaming_endpoint: _,
                    streaming_key: _,
                    public_url,
                } => StreamKind::Livestream { public_url },
            },
        }
    }
}
