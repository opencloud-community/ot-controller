// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;
use crate::utils::ExampleData;

/// Type of event resource.
///
/// Is used as type discriminator in field `type`.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(EventType::example_data()))
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

impl ExampleData for EventType {
    fn example_data() -> Self {
        Self::Single
    }
}
