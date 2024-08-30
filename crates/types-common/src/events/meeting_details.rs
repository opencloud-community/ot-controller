// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[allow(unused_imports)]
use crate::imports::*;
use crate::{
    call_in::CallInInfo, rooms::invite_codes::InviteCode, streaming::StreamingLink,
    utils::ExampleData,
};

/// Details about meeting
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(MeetingDetails::example_data()))
)]
pub struct MeetingDetails {
    /// The invite code id of the event
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub invite_code_id: Option<InviteCode>,

    /// The call-in information for the event
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub call_in: Option<CallInInfo>,

    /// The links for accessing the stream
    #[cfg_attr(feature = "serde", serde(default))]
    pub streaming_links: Vec<StreamingLink>,
}

impl ExampleData for MeetingDetails {
    fn example_data() -> Self {
        Self {
            invite_code_id: Some(InviteCode::example_data()),
            call_in: Some(CallInInfo::example_data()),
            streaming_links: vec![StreamingLink::example_data()],
        }
    }
}
