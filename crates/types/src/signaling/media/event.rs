// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to signaling events in the `media` namespace

use crate::core::ParticipantId;

#[allow(unused_imports)]
use crate::imports::*;

use super::{command::Target, MediaSessionType, TrickleCandidate};

/// The direction of a media link
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum LinkDirection {
    /// Upstream direction
    Upstream,

    /// Downstream direction
    Downstream,
}

/// Events sent out by the `media` module
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case", tag = "message")
)]
pub enum MediaEvent {
    /// SDP Offer, renegotiate publish
    SdpOffer(Sdp),

    /// SDP Answer, start the publish/subscription
    SdpAnswer(Sdp),

    /// SDP Candidate, used for ICE negotiation
    SdpCandidate(SdpCandidate),

    /// SDP End of Candidate, used for ICE negotiation
    SdpEndOfCandidates(Source),

    /// Signals that a webrtc connection has been established
    WebrtcUp(Source),

    /// Signals that a webrtc connection has been disconnected/destryoed by janus
    ///
    /// This message can, but wont always be received when a participant disconnects
    WebrtcDown(Source),

    /// Signals the media status for a participant
    MediaStatus(MediaStatus),

    /// A webrtc connection experienced package loss
    WebrtcSlow(Link),

    /// A specific participant should be focused
    FocusUpdate(FocusUpdate),

    /// The participant is requested to mute themselves
    RequestMute(RequestMute),

    /// Presenter role has been granted to the participant
    PresenterGranted,

    /// Presenter role has been revoked from the participant
    PresenterRevoked,

    /// Contains a error about what request failed. See [`Error`]
    Error(Error),
}

/// Event signaling that the participant should be muted
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RequestMute {
    /// The issuer of the mute request
    pub issuer: ParticipantId,
    /// Flag to determine if the mute shall be forced or not
    pub force: bool,
}

impl From<RequestMute> for MediaEvent {
    fn from(value: RequestMute) -> Self {
        Self::RequestMute(value)
    }
}

/// Specification of a source for media events
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Source {
    /// The source of this message
    pub source: ParticipantId,

    /// The type of stream
    pub media_session_type: MediaSessionType,
}

impl From<Target> for Source {
    fn from(target: Target) -> Self {
        Self {
            source: target.target,
            media_session_type: target.media_session_type,
        }
    }
}

/// SDP offer event
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Sdp {
    /// The payload of the sdp message
    pub sdp: String,

    /// The source of the media being negotiated
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub source: Source,
}

/// SDP Candidate event
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SdpCandidate {
    /// The payload of the sdp message
    pub candidate: TrickleCandidate,

    /// The source of the media being negotiated
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub source: Source,
}

/// Media status for a participant
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MediaStatus {
    /// Source of the media
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub source: Source,

    /// The kind of media
    pub kind: String,

    /// Whether media from the participant is received
    pub receiving: bool,
}

/// Description of a media link
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Link {
    /// Direction of the media link
    pub direction: LinkDirection,

    /// Source of the media link
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub source: Source,
}

/// Event signaling to the participant whether a specific participant should be focused on
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FocusUpdate {
    /// Id of the participant to focus on
    pub focus: Option<ParticipantId>,
}

impl From<FocusUpdate> for MediaEvent {
    fn from(value: FocusUpdate) -> Self {
        Self::FocusUpdate(value)
    }
}

/// Errors from the `media` module namespace
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case", tag = "error")
)]
pub enum Error {
    /// Frontend provided an invalid SDP offer
    InvalidSdpOffer,

    /// An SDP answer could not be handled
    HandleSdpAnswer,

    /// Frontend provided an invalid SDP candidate
    InvalidCandidate,

    /// Frontend sent an invalid end of SDP candidates command
    InvalidEndOfCandidates,

    /// Frontend sent an invalid request for an SDP offer
    InvalidRequestOffer(Source),

    /// Frontend sent an invalid configure request
    InvalidConfigureRequest(Source),

    /// Insufficient permissions to perform a command
    PermissionDenied,
}

impl From<Error> for MediaEvent {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn sdp_offer() {
        let sdp_offer = MediaEvent::SdpOffer(Sdp {
            sdp: "v=0...".into(),
            source: Source {
                source: ParticipantId::nil(),
                media_session_type: MediaSessionType::Video,
            },
        });

        assert_eq!(
            serde_json::to_value(sdp_offer).unwrap(),
            json!({
                "message": "sdp_offer",
                "sdp": "v=0...",
                "source": "00000000-0000-0000-0000-000000000000",
                "media_session_type": "video"
            })
        );
    }

    #[test]
    fn sdp_answer() {
        let sdp_answer = MediaEvent::SdpAnswer(Sdp {
            sdp: "v=0...".into(),
            source: Source {
                source: ParticipantId::nil(),
                media_session_type: MediaSessionType::Video,
            },
        });

        assert_eq!(
            serde_json::to_value(sdp_answer).unwrap(),
            json!({
                "message": "sdp_answer",
                "sdp": "v=0...",
                "source": "00000000-0000-0000-0000-000000000000",
                "media_session_type": "video"
            })
        );
    }

    #[test]
    fn sdp_candidate() {
        let sdp_candidate = MediaEvent::SdpCandidate(SdpCandidate {
            candidate: TrickleCandidate {
                sdp_m_line_index: 1,
                candidate: "candidate:4 1 UDP 123456 192.168.178.1 123456 typ host".into(),
            },
            source: Source {
                source: ParticipantId::nil(),
                media_session_type: MediaSessionType::Video,
            },
        });

        assert_eq!(
            serde_json::to_value(sdp_candidate).unwrap(),
            json!({
              "message": "sdp_candidate",
              "candidate": {
                  "sdpMLineIndex": 1,
                  "candidate": "candidate:4 1 UDP 123456 192.168.178.1 123456 typ host"
              },
              "source": "00000000-0000-0000-0000-000000000000",
              "media_session_type": "video"
            })
        );
    }

    #[test]
    fn test_webrtc_up() {
        let webrtc_up = MediaEvent::WebrtcUp(Source {
            source: ParticipantId::nil(),
            media_session_type: MediaSessionType::Video,
        });

        assert_eq!(
            serde_json::to_value(webrtc_up).unwrap(),
            json!({
                "message": "webrtc_up",
                "source": "00000000-0000-0000-0000-000000000000",
                "media_session_type": "video"
            })
        );
    }

    #[test]
    fn test_webrtc_down() {
        let webrtc_down = MediaEvent::WebrtcDown(Source {
            source: ParticipantId::nil(),
            media_session_type: MediaSessionType::Video,
        });

        assert_eq!(
            serde_json::to_value(webrtc_down).unwrap(),
            json!({
                "message": "webrtc_down",
                "source": "00000000-0000-0000-0000-000000000000",
                "media_session_type": "video"
            })
        );
    }

    #[test]
    fn test_media_status() {
        let webrtc_down = MediaEvent::MediaStatus(MediaStatus {
            source: Source {
                source: ParticipantId::nil(),
                media_session_type: MediaSessionType::Video,
            },
            kind: "video".to_owned(),
            receiving: true,
        });

        assert_eq!(
            serde_json::to_value(webrtc_down).unwrap(),
            json!({
                "message": "media_status",
                "source": "00000000-0000-0000-0000-000000000000",
                "media_session_type": "video",
                "kind": "video",
                "receiving": true
            })
        );
    }

    #[test]
    fn test_webrtc_slow() {
        let web_rtc_slow = MediaEvent::WebrtcSlow(Link {
            direction: LinkDirection::Upstream,
            source: Source {
                source: ParticipantId::nil(),
                media_session_type: MediaSessionType::Video,
            },
        });

        assert_eq!(
            serde_json::to_value(web_rtc_slow).unwrap(),
            json!({
                "message": "webrtc_slow",
                "direction": "upstream",
                "source": "00000000-0000-0000-0000-000000000000",
                "media_session_type": "video"
            })
        );
    }

    #[test]
    fn test_request_mute() {
        let request_mute = MediaEvent::RequestMute(RequestMute {
            issuer: ParticipantId::nil(),
            force: false,
        });

        assert_eq!(
            serde_json::to_value(request_mute).unwrap(),
            json!({
                "message": "request_mute",
                "issuer": "00000000-0000-0000-0000-000000000000",
                "force": false
            })
        );
    }

    #[test]
    fn test_errors() {
        let errors_and_expected = vec![
            (
                Error::InvalidSdpOffer,
                json!({"error": "invalid_sdp_offer"}),
            ),
            (
                Error::HandleSdpAnswer,
                json!({"error": "handle_sdp_answer"}),
            ),
            (
                Error::InvalidCandidate,
                json!({"error": "invalid_candidate"}),
            ),
            (
                Error::InvalidEndOfCandidates,
                json!({"error": "invalid_end_of_candidates"}),
            ),
            (
                Error::InvalidRequestOffer(Source {
                    source: ParticipantId::nil(),
                    media_session_type: MediaSessionType::Video,
                }),
                json!({
                    "error": "invalid_request_offer",
                    "source": "00000000-0000-0000-0000-000000000000",
                    "media_session_type": "video",
                }),
            ),
            (
                Error::InvalidConfigureRequest(Source {
                    source: ParticipantId::nil(),
                    media_session_type: MediaSessionType::Video,
                }),
                json!({
                    "error": "invalid_configure_request",
                    "source": "00000000-0000-0000-0000-000000000000",
                    "media_session_type": "video"
                }),
            ),
        ];

        for (error, expected) in errors_and_expected {
            let produced = serde_json::to_value(error).unwrap();
            println!("{produced}");
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn presenter_granted() {
        let presenter_granted = MediaEvent::PresenterGranted;

        assert_eq!(
            serde_json::to_value(presenter_granted).unwrap(),
            json!({
                "message": "presenter_granted"
            })
        );
    }

    #[test]
    fn presenter_revoked() {
        let presenter_revoked = MediaEvent::PresenterRevoked;

        assert_eq!(
            serde_json::to_value(presenter_revoked).unwrap(),
            json!({
                "message": "presenter_revoked"
            })
        );
    }
}
