// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::settings_file;

/// The runtime setting of a TURN server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnServer {
    /// The URIs for the TURN server in RFC7065 format.
    pub uris: Vec<String>,

    /// A pre-shared key for the TURN server.
    pub pre_shared_key: String,
}

impl From<settings_file::TurnServer> for TurnServer {
    fn from(
        settings_file::TurnServer {
            uris,
            pre_shared_key,
        }: settings_file::TurnServer,
    ) -> Self {
        Self {
            uris,
            pre_shared_key,
        }
    }
}
