// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::users::GroupName;
use opentalk_types_signaling::ParticipantId;

#[allow(unused_imports)]
use crate::imports::*;

/// Specifies if the chat message is global, private or group message
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "scope", content = "target", rename_all = "snake_case")
)]
pub enum Scope {
    /// Global scope for chat
    Global,

    /// Group scope for chat
    Group(GroupName),

    /// Private scope for chat
    Private(ParticipantId),
}
