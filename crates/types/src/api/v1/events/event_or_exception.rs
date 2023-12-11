// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;

use super::{EventExceptionResource, EventResource};

/// Return type of the `GET /events` endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(untagged))]
#[allow(clippy::large_enum_variant)]
pub enum EventOrException {
    /// Event resource
    Event(EventResource),
    /// Event exception resource
    Exception(EventExceptionResource),
}
