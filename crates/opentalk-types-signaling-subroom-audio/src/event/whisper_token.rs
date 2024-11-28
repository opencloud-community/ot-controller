// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::whisper_id::WhisperId;

/// A livekit access token to a whisper group
///
/// The token is issued for the associated whisper room and restricted to audio-only.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WhisperToken {
    /// Unique id for the whisper group
    pub whisper_id: WhisperId,
    /// The JWT access token for the whisper room
    pub token: String,
}
