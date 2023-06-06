// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use db_storage::users::User;
use serde::{Deserialize, Serialize};
use types::core::UserId;

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
