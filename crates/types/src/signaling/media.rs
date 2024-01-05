// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `media` namespace

mod media_session_state;
mod media_session_type;
mod trickle_candidate;

pub mod command;
pub mod event;
pub mod peer_state;
pub mod state;

use std::collections::HashMap;

pub use media_session_state::MediaSessionState;
pub use media_session_type::{MediaSessionType, MediaSessionTypeParseError};
pub use trickle_candidate::TrickleCandidate;

/// The media state of a participant
pub type ParticipantMediaState = HashMap<MediaSessionType, MediaSessionState>;

/// The namespace string for the signaling module
pub const NAMESPACE: &str = "media";
