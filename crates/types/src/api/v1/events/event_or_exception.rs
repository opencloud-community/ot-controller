// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::{EventExceptionResource, EventResource};
#[allow(unused_imports)]
use crate::imports::*;
use crate::utils::ExampleData;

/// Return type of the `GET /events` endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(untagged))]
#[allow(clippy::large_enum_variant)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(EventOrException::example_data()))
)]
pub enum EventOrException {
    /// Event resource
    Event(EventResource),
    /// Event exception resource
    Exception(EventExceptionResource),
}

impl ExampleData for EventOrException {
    fn example_data() -> Self {
        Self::Event(EventResource::example_data())
    }
}
