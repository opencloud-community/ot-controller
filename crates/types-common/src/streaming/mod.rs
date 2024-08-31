// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Data types for handling streaming.

mod streaming_key;
mod streaming_kind;
mod streaming_link;
mod streaming_target_id;
mod streaming_target_kind;

pub use streaming_key::StreamingKey;
pub use streaming_kind::{StreamingKind, StreamingKindType};
pub use streaming_link::StreamingLink;
pub use streaming_target_id::StreamingTargetId;
pub use streaming_target_kind::StreamingTargetKind;
