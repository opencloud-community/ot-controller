// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `shared_folder` namespace

use crate::{common::shared_folder::SharedFolder, signaling::Role};

#[allow(unused_imports)]
use crate::imports::*;

/// Events sent out by the `shared_folder` module
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "message", rename_all = "snake_case")
)]
pub enum SharedFolderEvent {
    /// The shared folder data has changed, e.g. by a participant
    /// being promoted to or demoted from moderator role
    Updated(SharedFolder),
}

impl SharedFolderEvent {
    /// Get an equivalent shared folder event, cut down to match the signaling role
    pub fn for_signaling_role(self, role: Role) -> Self {
        match self {
            SharedFolderEvent::Updated(state) => {
                SharedFolderEvent::Updated(state.for_signaling_role(role))
            }
        }
    }
}
