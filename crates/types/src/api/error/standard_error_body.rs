// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::borrow::Cow;

#[allow(unused_imports)]
use crate::imports::*;

/// Standard API error body
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StandardErrorBody {
    /// Machine readable error code
    pub code: Cow<'static, str>,

    /// Human readable message
    pub message: Cow<'static, str>,
}
