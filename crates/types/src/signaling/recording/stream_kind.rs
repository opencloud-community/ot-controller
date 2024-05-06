// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use url::Url;

use crate::common::streaming::StreamingTargetKind;
#[allow(unused_imports)]
use crate::imports::*;

/// The kind of the stream
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
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

/// The kind of the stream
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "streaming_kind", rename_all = "snake_case")
)]
pub enum StreamKindSecret {
    /// Recording kind
    Recording,
    /// Livestream kind
    Livestream(StreamingTargetKind),
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
