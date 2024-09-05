// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Body of the join command
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Join {
    /// The users display name
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none",)
    )]
    pub display_name: Option<String>,
}
