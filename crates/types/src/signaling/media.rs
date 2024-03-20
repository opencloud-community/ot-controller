// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `media` namespace

mod media_session_state;
mod media_session_type;
mod participant_speaking_state;
mod speaking_state;
mod trickle_candidate;
mod update_speaking_state;

pub mod command;
pub mod event;
pub mod peer_state;
pub mod state;

pub use media_session_state::MediaSessionState;
pub use media_session_type::{MediaSessionType, MediaSessionTypeParseError};
pub use participant_speaking_state::ParticipantSpeakingState;
pub use speaking_state::SpeakingState;
pub use trickle_candidate::TrickleCandidate;
pub use update_speaking_state::UpdateSpeakingState;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The media state of a participant
#[derive(Debug, Default, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ParticipantMediaState {
    /// See [`MediaSessionType::Video`]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub video: Option<MediaSessionState>,

    /// See [`MediaSessionType::Screen`]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub screen: Option<MediaSessionState>,
}

impl ParticipantMediaState {
    /// Insert a new state for a media session type, returning the old state
    pub fn insert(
        &mut self,
        media_session_type: MediaSessionType,
        state: MediaSessionState,
    ) -> Option<MediaSessionState> {
        match media_session_type {
            MediaSessionType::Video => self.video.replace(state),
            MediaSessionType::Screen => self.screen.replace(state),
        }
    }

    /// Get the media session state for the given type
    pub fn get(&self, media_session_type: MediaSessionType) -> Option<MediaSessionState> {
        match media_session_type {
            MediaSessionType::Video => self.video,
            MediaSessionType::Screen => self.screen,
        }
    }

    /// Get a mutable reference to the media session state for the given type
    pub fn get_mut(
        &mut self,
        media_session_type: MediaSessionType,
    ) -> Option<&mut MediaSessionState> {
        match media_session_type {
            MediaSessionType::Video => self.video.as_mut(),
            MediaSessionType::Screen => self.screen.as_mut(),
        }
    }

    /// Remove the media session state for the given type
    pub fn remove(&mut self, media_session_type: MediaSessionType) -> Option<MediaSessionState> {
        match media_session_type {
            MediaSessionType::Video => self.video.take(),
            MediaSessionType::Screen => self.screen.take(),
        }
    }
}

/// The namespace string for the signaling module
pub const NAMESPACE: &str = "media";

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_with_video_and_screen() {
        use serde_json::json;

        let state = ParticipantMediaState {
            video: Some(MediaSessionState {
                video: true,
                audio: true,
            }),
            screen: Some(MediaSessionState {
                video: true,
                audio: false,
            }),
        };

        let expected = json!({
            "video": {
                "video": true,
                "audio": true,
            },
            "screen": {
                "video": true,
                "audio": false,
            },
        });

        assert_eq!(serde_json::to_value(state).unwrap(), expected);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_with_video() {
        use serde_json::json;

        let state = ParticipantMediaState {
            video: Some(MediaSessionState {
                video: true,
                audio: true,
            }),
            screen: None,
        };

        let expected = json!({
            "video": {
                "video": true,
                "audio": true,
            },
        });

        assert_eq!(serde_json::to_value(state).unwrap(), expected);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_with_screen() {
        use serde_json::json;

        let state = ParticipantMediaState {
            video: None,
            screen: Some(MediaSessionState {
                video: true,
                audio: false,
            }),
        };

        let expected = json!({
            "screen": {
                "video": true,
                "audio": false,
            },
        });

        assert_eq!(serde_json::to_value(state).unwrap(), expected);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_without_media() {
        use serde_json::json;

        let state = ParticipantMediaState {
            video: None,
            screen: None,
        };

        let expected = json!({});

        assert_eq!(serde_json::to_value(state).unwrap(), expected);
    }
}
