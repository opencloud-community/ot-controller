// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `shared_folder` namespace

use opentalk_types_common::shared_folders::SharedFolder;
use opentalk_types_signaling::{ForRole, Role};

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

impl ForRole for SharedFolderEvent {
    /// Get an equivalent shared folder event, cut down to match the signaling role
    fn for_role(self, role: Role) -> Self {
        match self {
            SharedFolderEvent::Updated(state) => SharedFolderEvent::Updated(state.for_role(role)),
        }
    }
}
