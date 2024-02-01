// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

/// Body for the `DELETE /events/{event_id}/shared_folder` endpoint
#[derive(Default, Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DeleteSharedFolderQuery {
    /// Flag to force delete the reference to the shared folder if the deletion of the shared folder fails
    #[cfg_attr(feature = "serde", serde(default))]
    pub force_delete_reference_if_shared_folder_deletion_fails: bool,
}
