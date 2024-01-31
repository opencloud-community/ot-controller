// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::core::InviteCodeId;
#[allow(unused_imports)]
use crate::imports::*;

/// Verify body for *POST /invite/verify*
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, Validate))]
pub struct PostInviteVerifyRequestBody {
    /// The invite code id
    pub invite_code: InviteCodeId,
}
