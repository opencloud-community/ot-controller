// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

/// Body for the `PUT /events/{event_id}/shared_folder` endpoint
#[derive(Default, Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::IntoParams))]
pub struct PutSharedFolderQuery {
    /// Flag to disable email notification
    #[cfg_attr(feature = "serde", serde(default))]
    pub suppress_email_notification: bool,
}
