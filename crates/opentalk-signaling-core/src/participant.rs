// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage::users::User;
use opentalk_types_common::users::UserId;
use opentalk_types_signaling::ParticipationKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Participant<U> {
    User(U),
    Guest,
    Sip,
    Recorder,
}

impl<U> Participant<U> {
    pub fn as_kind_str(&self) -> &'static str {
        match self {
            Participant::User(_) => "user",
            Participant::Guest => "guest",
            Participant::Sip => "sip",
            Participant::Recorder => "recorder",
        }
    }

    pub fn kind(&self) -> ParticipationKind {
        match self {
            Participant::User(_) => ParticipationKind::User,
            Participant::Guest => ParticipationKind::Guest,
            Participant::Sip => ParticipationKind::Sip,
            Participant::Recorder => ParticipationKind::Recorder,
        }
    }
}

impl From<UserId> for Participant<UserId> {
    fn from(id: UserId) -> Self {
        Participant::User(id)
    }
}

impl From<User> for Participant<User> {
    fn from(user: User) -> Self {
        Participant::User(user)
    }
}

impl Participant<User> {
    /// Returns the UserId when the participant is a registered user
    pub fn user_id(&self) -> Option<UserId> {
        match self {
            Participant::User(user) => Some(user.id),
            Participant::Guest => None,
            Participant::Sip => None,
            Participant::Recorder => None,
        }
    }
}
