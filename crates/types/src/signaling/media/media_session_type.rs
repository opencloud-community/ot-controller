// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use snafu::Snafu;

#[allow(unused_imports)]
use crate::imports::*;

/// The type of media session
#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MediaSessionType {
    /// Participant publishes "video" media, which is moving picture with lower quality and higher framerate (e.g. webcam)
    #[cfg_attr(feature = "serde", serde(rename = "video"))]
    Video,

    /// Participant publishes "screen" media, which has a higher resolution but lower framerate (e.g. desktop/presentations)
    #[cfg_attr(feature = "serde", serde(rename = "screen"))]
    Screen,
}

impl MediaSessionType {
    /// Return the string slice representing the media session type
    pub fn as_str(&self) -> &'static str {
        match self {
            MediaSessionType::Video => "video",
            MediaSessionType::Screen => "screen",
        }
    }
}

/// Error when attempting to parse a [`MediaSessionType`]
///
/// Derived using [`snafu::Snafu`]
#[derive(Debug, Snafu)]
#[snafu(display("Invalid media session type, {value}"))]
pub struct MediaSessionTypeParseError {
    value: u64,
}

impl TryFrom<u64> for MediaSessionType {
    type Error = MediaSessionTypeParseError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Video),
            2 => Ok(Self::Screen),
            _ => Err(MediaSessionTypeParseError { value }),
        }
    }
}

impl From<MediaSessionType> for u64 {
    fn from(value: MediaSessionType) -> Self {
        match value {
            MediaSessionType::Video => 1,
            MediaSessionType::Screen => 2,
        }
    }
}

impl std::fmt::Display for MediaSessionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Into::<u64>::into(*self).fmt(f)
    }
}
