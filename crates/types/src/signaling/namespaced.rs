// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{modules::ModuleId, time::Timestamp};

#[allow(unused_imports)]
use crate::imports::*;

/// An envelope of a command annotated with their respective module id.
///
/// This is used for WebSocket messages sent to the backend.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct NamespacedCommand<O> {
    /// The module to which the message is targeted
    pub module: ModuleId,
    /// The payload of the message
    pub payload: O,
}

/// An envelope of an event annotated with their respective module id.
///
/// This is used for WebSocket messages sent to the frontend.
/// Similar to [`NamespacedCommand`], but includes a timestamp field.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct NamespacedEvent<O> {
    /// The namespace to which the message is targeted
    pub module: ModuleId,
    /// The timestamp indicating the creation of the message
    pub timestamp: Timestamp,
    /// The payload of the message
    pub payload: O,
}
