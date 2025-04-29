// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::users::DisplayName;
use opentalk_types_signaling_legal_vote::cancel::CancelReason;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ResolvedCancel {
    /// The user id of the entry issuer
    pub user: DisplayName,
    /// The reason for the cancel
    #[serde(flatten)]
    pub reason: CancelReason,
}
