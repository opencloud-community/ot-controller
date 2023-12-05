// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

/// Type of event resource.
///
/// Is used as type discriminator in field `type`.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum EventType {
    /// Single event
    Single,
    /// Recurring event
    Recurring,
    /// Event instance
    Instance,
    /// Event exception
    Exception,
}
