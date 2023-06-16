// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use janus_client::{Jsep, TrickleCandidate};
use types::{
    core::ParticipantId,
    signaling::media::{command::SubscriberConfiguration, event::Source, MediaSessionType},
};

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum Request {
    RequestOffer { without_video: bool },
    SdpOffer(String),
    SdpAnswer(String),
    Candidate(TrickleCandidate),
    EndOfCandidates,
    PublisherConfigure(PublishConfiguration),
    SubscriberConfigure(SubscriberConfiguration),
}

#[derive(Debug)]
pub struct PublishConfiguration {
    pub video: bool,
    pub audio: bool,
}

#[derive(Debug)]
pub enum Response {
    SdpAnswer(Jsep),
    SdpOffer(Jsep),
    None,
}

/// Used to relay messages to the WebSocket
#[derive(Debug)]
pub enum WebRtcEvent {
    WebRtcUp,
    WebRtcDown,
    Media(Media),
    SlowLink(LinkDirection),
    Trickle(TrickleMessage),
    AssociatedMcuDied,
    StartedTalking,
    StoppedTalking,
}

#[derive(Debug)]
pub struct Media {
    pub kind: String,
    pub receiving: bool,
}

impl From<janus_client::incoming::Media> for Media {
    fn from(value: janus_client::incoming::Media) -> Self {
        Self {
            kind: value.kind,
            receiving: value.receiving,
        }
    }
}

#[derive(Debug)]
pub enum LinkDirection {
    Upstream,
    Downstream,
}

#[derive(Debug)]
pub enum TrickleMessage {
    Completed,
    Candidate(TrickleCandidate),
}

/// Key Type consisting of ParticipantID and MediaSessionType
///
/// Used as a key to be able to support multiple media sessions per participant.
/// Used in the description of a Janus room as fallback.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MediaSessionKey(pub ParticipantId, pub MediaSessionType);

impl From<MediaSessionKey> for Source {
    fn from(media_session_key: MediaSessionKey) -> Self {
        Self {
            source: media_session_key.0,
            media_session_type: media_session_key.1,
        }
    }
}

/// We use this mapping in the description of a Janus room
///
/// For this we need to be able to convert it into a String.
impl std::fmt::Display for MediaSessionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}", self.0, self.1)
    }
}
