// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum Event {
    Credentials(Credentials),
    Error(Error),
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Credentials {
    pub room: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum Error {
    LivekitUnavailable,
}
