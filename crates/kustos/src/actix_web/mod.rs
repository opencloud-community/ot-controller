// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod middleware;

pub use middleware::KustosService;
#[derive(Debug, Clone)]
pub struct User(uuid::Uuid);

impl From<uuid::Uuid> for User {
    fn from(uuid: uuid::Uuid) -> Self {
        Self(uuid)
    }
}

#[derive(Debug, Clone)]
pub struct Invite(uuid::Uuid);

impl From<uuid::Uuid> for Invite {
    fn from(uuid: uuid::Uuid) -> Self {
        Self(uuid)
    }
}
