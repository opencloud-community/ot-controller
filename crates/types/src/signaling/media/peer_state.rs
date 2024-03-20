// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Peer frontend data for `recording` namespace

#[allow(unused_imports)]
use crate::imports::*;

use super::ParticipantMediaState;

/// The state of other participants in the `recording` module.
///
/// This struct is sent to the participant in the `join_success` message
/// which will contain this information for each participant in the meeting.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MediaPeerState {
    /// The media state of the peer
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub state: ParticipantMediaState,

    /// Whether the participant has permission to share the screen
    pub is_presenter: bool,
}

#[cfg(feature = "serde")]
impl SignalingModulePeerFrontendData for MediaPeerState {
    const NAMESPACE: Option<&'static str> = Some(super::NAMESPACE);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_with_video_and_screen() {
        use serde_json::json;

        use crate::signaling::media::MediaSessionState;

        let state = MediaPeerState {
            state: ParticipantMediaState {
                video: Some(MediaSessionState {
                    video: true,
                    audio: true,
                }),
                screen: Some(MediaSessionState {
                    video: true,
                    audio: false,
                }),
            },
            is_presenter: true,
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
            "is_presenter": true,
        });

        assert_eq!(serde_json::to_value(state).unwrap(), expected);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_with_video() {
        use serde_json::json;

        use crate::signaling::media::MediaSessionState;

        let state = MediaPeerState {
            state: ParticipantMediaState {
                video: Some(MediaSessionState {
                    video: true,
                    audio: true,
                }),
                screen: None,
            },
            is_presenter: false,
        };

        let expected = json!({
            "video": {
                "video": true,
                "audio": true,
            },
            "is_presenter": false,
        });

        assert_eq!(serde_json::to_value(state).unwrap(), expected);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_without_media() {
        use serde_json::json;

        let state = MediaPeerState {
            state: ParticipantMediaState {
                video: None,
                screen: None,
            },
            is_presenter: false,
        };

        let expected = json!({
            "is_presenter": false,
        });

        assert_eq!(serde_json::to_value(state).unwrap(), expected);
    }
}
