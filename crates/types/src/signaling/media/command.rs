// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `media` namespace

use crate::core::ParticipantId;

#[allow(unused_imports)]
use crate::imports::*;

use super::{MediaSessionState, MediaSessionType, TrickleCandidate};

/// Commands received by the `media` module
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "action", rename_all = "snake_case")
)]
pub enum MediaCommand {
    /// The participant successfully established a stream
    PublishComplete(MediaSessionInfo),

    /// The participants publish stream has stopped (for whatever reason)
    Unpublish(AssociatedMediaSession),

    /// The participant updates its stream-state
    ///
    /// This can be mute/unmute of video or audio
    UpdateMediaSession(MediaSessionInfo),

    /// A moderators request to mute one or more participants
    ModeratorMute(RequestMute),

    /// SDP offer
    Publish(TargetedSdp),

    /// SDP Answer
    SdpAnswer(TargetedSdp),

    /// SDP Candidate
    SdpCandidate(TargetedCandidate),

    /// SDP EndOfCandidate
    SdpEndOfCandidates(Target),

    /// SDP request offer
    Subscribe(TargetSubscribe),

    /// Restart an existing subscription
    Resubscribe(Target),

    /// Grant the presenter role for a set of participants
    GrantPresenterRole(ParticipantSelection),

    /// Revoke the presenter role for a set of participants
    RevokePresenterRole(ParticipantSelection),

    /// SDP request to configure subscription
    Configure(TargetConfigure),
}

/// Information about a media session
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MediaSessionInfo {
    /// The stream type that has been published
    pub media_session_type: MediaSessionType,

    /// The current state of the session
    pub media_session_state: MediaSessionState,
}

/// An established media session
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AssociatedMediaSession {
    /// The stream type that has been published
    pub media_session_type: MediaSessionType,
}

/// Specification of a target for media commands
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Target {
    /// The target of this message.
    ///
    /// If the own ID is specified it is used to negotiate the publish stream.
    pub target: ParticipantId,

    /// The type of stream
    pub media_session_type: MediaSessionType,
}

/// Request a number of participants to mute themselves
///
/// May only be processed if the issuer is a moderator
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RequestMute {
    /// Participants that shall be muted
    pub targets: Vec<ParticipantId>,

    /// Force mute the participant(s)
    pub force: bool,
}

/// Targeted SDP offer or answer
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TargetedSdp {
    /// The payload of the sdp message
    pub sdp: String,

    /// The target of this SDP message.
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub target: Target,
}

/// Command to inform the backend about a SDP candidate
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TargetedCandidate {
    /// The payload of the sdp message
    pub candidate: TrickleCandidate,

    /// The target of this Candidate
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub target: Target,
}

/// Command to subscribe for a target
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TargetSubscribe {
    /// The target of the subscription
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub target: Target,

    /// Do not subscribe to the video stream.
    /// Primarily used for SIP.
    #[cfg_attr(feature = "serde", serde(default))]
    pub without_video: bool,
}

/// Give a list of participants write access to the protocol
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ParticipantSelection {
    /// The targeted participants
    pub participant_ids: Vec<ParticipantId>,
}

/// Command to configure a target subscription
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TargetConfigure {
    /// The target of this configure
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub target: Target,

    /// New Configuration
    ///
    /// Contains the configuration changes/settings to be applied.
    pub configuration: SubscriberConfiguration,
}

/// Configuration of a video subscription
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SubscriberConfiguration {
    /// Video Feed
    ///
    /// If true, will configure the connection to receive the video stream.
    /// If false, will disable the video feed relaying.
    pub video: Option<bool>,

    /// Substream
    ///
    /// If enabled, the selected substream of the three (0-2) available
    pub substream: Option<u8>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{core::ParticipantId, signaling::media::MediaSessionType};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn publish() {
        let json = json!({
            "action": "publish_complete",
            "media_session_type": "video",
            "media_session_state": {
                "audio": false,
                "video": false,
            },
        });

        let msg: MediaCommand = serde_json::from_value(json).unwrap();

        if let MediaCommand::PublishComplete(MediaSessionInfo {
            media_session_type,
            media_session_state,
        }) = msg
        {
            assert_eq!(media_session_type, MediaSessionType::Video);
            assert!(!media_session_state.audio);
            assert!(!media_session_state.video);
        } else {
            panic!()
        }
    }

    #[test]
    fn unpublish() {
        let json = json!({
            "action": "unpublish",
            "media_session_type": "video",
        });

        let msg: MediaCommand = serde_json::from_value(json).unwrap();

        if let MediaCommand::Unpublish(AssociatedMediaSession { media_session_type }) = msg {
            assert_eq!(media_session_type, MediaSessionType::Video);
        } else {
            panic!()
        }
    }

    #[test]
    fn update_media_session() {
        let json = json!({
            "action": "update_media_session",
            "media_session_type": "video",
            "media_session_state": {
                "audio": true,
                "video": false,
            },
        });

        let msg: MediaCommand = serde_json::from_value(json).unwrap();

        if let MediaCommand::UpdateMediaSession(MediaSessionInfo {
            media_session_type,
            media_session_state,
        }) = msg
        {
            assert_eq!(media_session_type, MediaSessionType::Video);
            assert!(media_session_state.audio);
            assert!(!media_session_state.video);
        } else {
            panic!()
        }
    }

    #[test]
    fn moderator_mute_single() {
        let json = json!({
            "action": "moderator_mute",
            "targets": ["00000000-0000-0000-0000-000000000000"],
            "force": true,
        });

        let msg: MediaCommand = serde_json::from_value(json).unwrap();

        if let MediaCommand::ModeratorMute(RequestMute { targets, force }) = msg {
            assert_eq!(targets, vec![ParticipantId::nil()]);
            assert!(force);
        } else {
            panic!()
        }
    }

    #[test]
    fn moderator_mute_many() {
        let json = json!({
            "action": "moderator_mute",
            "targets": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-000000000001", "00000000-0000-0000-0000-000000000002"],
            "force": false,
        });

        let msg: MediaCommand = serde_json::from_value(json).unwrap();

        if let MediaCommand::ModeratorMute(RequestMute { targets, force }) = msg {
            assert_eq!(
                targets,
                [
                    ParticipantId::from_u128(0),
                    ParticipantId::from_u128(1),
                    ParticipantId::from_u128(2)
                ]
            );
            assert!(!force);
        } else {
            panic!()
        }
    }

    #[test]
    fn offer() {
        let json = json!({
            "action": "publish",
            "sdp": "v=0\r\n...",
            "target": "00000000-0000-0000-0000-000000000000",
            "media_session_type": "video",
        });

        let msg: MediaCommand = serde_json::from_value(json).unwrap();

        if let MediaCommand::Publish(TargetedSdp {
            sdp,
            target:
                Target {
                    target,
                    media_session_type,
                },
        }) = msg
        {
            assert_eq!(sdp, "v=0\r\n...");
            assert_eq!(target, ParticipantId::nil());
            assert_eq!(media_session_type, MediaSessionType::Video);
        } else {
            panic!()
        }
    }

    #[test]
    fn answer() {
        let json = json!({
            "action": "sdp_answer",
            "sdp": "v=0\r\n...",
            "target": "00000000-0000-0000-0000-000000000000",
            "media_session_type": "video",
        });

        let msg: MediaCommand = serde_json::from_value(json).unwrap();

        if let MediaCommand::SdpAnswer(TargetedSdp {
            sdp,
            target:
                Target {
                    target,
                    media_session_type,
                },
        }) = msg
        {
            assert_eq!(sdp, "v=0\r\n...");
            assert_eq!(target, ParticipantId::nil());
            assert_eq!(media_session_type, MediaSessionType::Video);
        } else {
            panic!()
        }
    }

    #[test]
    fn candidate() {
        let json = json!({
            "action": "sdp_candidate",
            "candidate": {
                "candidate": "candidate:4 1 UDP 123456 192.168.178.1 123456 typ host",
                "sdpMLineIndex": 1,
            },
            "target": "00000000-0000-0000-0000-000000000000",
            "media_session_type": "video",
        });

        let msg: MediaCommand = serde_json::from_value(json).unwrap();

        if let MediaCommand::SdpCandidate(TargetedCandidate {
            candidate:
                TrickleCandidate {
                    sdp_m_line_index,
                    candidate,
                },
            target:
                Target {
                    target,
                    media_session_type,
                },
        }) = msg
        {
            assert_eq!(sdp_m_line_index, 1);
            assert_eq!(
                candidate,
                "candidate:4 1 UDP 123456 192.168.178.1 123456 typ host"
            );
            assert_eq!(target, ParticipantId::nil());
            assert_eq!(media_session_type, MediaSessionType::Video);
        } else {
            panic!()
        }
    }

    #[test]
    fn end_of_candidates() {
        let json = json!({
            "action": "sdp_end_of_candidates",
            "target": "00000000-0000-0000-0000-000000000000",
            "media_session_type": "video",
        });

        let msg: MediaCommand = serde_json::from_value(json).unwrap();

        if let MediaCommand::SdpEndOfCandidates(Target {
            target,
            media_session_type,
        }) = msg
        {
            assert_eq!(target, ParticipantId::nil());
            assert_eq!(media_session_type, MediaSessionType::Video);
        } else {
            panic!()
        }
    }

    #[test]
    fn request_offer() {
        let json = json!({
            "action": "subscribe",
            "target": "00000000-0000-0000-0000-000000000000",
            "media_session_type": "video",
        });

        let msg: MediaCommand = serde_json::from_value(json).unwrap();

        if let MediaCommand::Subscribe(TargetSubscribe {
            target:
                Target {
                    target,
                    media_session_type,
                },
            without_video,
        }) = msg
        {
            assert_eq!(target, ParticipantId::nil());
            assert_eq!(media_session_type, MediaSessionType::Video);
            assert!(!without_video);
        } else {
            panic!()
        }
    }

    #[test]
    fn grant_presenter_role() {
        let json = json!({
            "action": "grant_presenter_role",
            "participant_ids": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-000000000000"],
        });

        let msg: MediaCommand = serde_json::from_value(json).unwrap();

        if let MediaCommand::GrantPresenterRole(ParticipantSelection { participant_ids }) = msg {
            assert_eq!(
                participant_ids,
                vec![ParticipantId::nil(), ParticipantId::nil()]
            );
        } else {
            panic!()
        }
    }

    #[test]
    fn revoke_presenter_role() {
        let json = json!({
            "action": "revoke_presenter_role",
            "participant_ids": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-000000000000"],
        });

        let msg: MediaCommand = serde_json::from_value(json).unwrap();

        if let MediaCommand::RevokePresenterRole(ParticipantSelection { participant_ids }) = msg {
            assert_eq!(
                participant_ids,
                vec![ParticipantId::nil(), ParticipantId::nil()]
            );
        } else {
            panic!()
        }
    }
}
