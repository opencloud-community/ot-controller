// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

/// State of a media session
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MediaSessionState {
    /// Video is enabled for the session
    pub video: bool,

    /// Audio is enabled for the session
    pub audio: bool,
}

impl MediaSessionState {
    /// Construct a MediaSessionState with all values set to true
    #[must_use]
    pub fn audio_and_video() -> Self {
        Self {
            audio: true,
            video: true,
        }
    }
    /// Construct a MediaSessionState with only audio set to true
    #[must_use]
    pub fn audio() -> Self {
        Self {
            audio: true,
            video: false,
        }
    }
    /// Construct a MediaSessionState with only video set to true
    #[must_use]
    pub fn video() -> Self {
        Self {
            audio: false,
            video: true,
        }
    }
}

impl std::fmt::Display for MediaSessionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaSessionState {
                video: true,
                audio: true,
            } => write!(f, "video+audio"),
            MediaSessionState {
                video: true,
                audio: false,
            } => write!(f, "video only"),
            MediaSessionState {
                video: false,
                audio: true,
            } => write!(f, "audio only"),
            MediaSessionState {
                video: false,
                audio: false,
            } => write!(f, "none"),
        }
    }
}
