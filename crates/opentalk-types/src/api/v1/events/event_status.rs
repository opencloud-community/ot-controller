// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::utils::ExampleData;

#[allow(unused_imports)]
use crate::imports::*;

/// Status of an event
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(EventStatus::example_data()))
)]
pub enum EventStatus {
    /// Default status, event is ok
    Ok,

    /// Event (or event instance) was cancelled
    Cancelled,
}

impl ExampleData for EventStatus {
    fn example_data() -> Self {
        Self::Ok
    }
}
