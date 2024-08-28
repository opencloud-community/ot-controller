// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, TimeZone as _, Utc};

#[allow(unused_imports)]
use crate::imports::*;
use crate::utils::ExampleData;

/// Body for *POST /rooms/{room_id}/invites*
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature="utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(PostInviteRequestBody::example_data()))
)]
pub struct PostInviteRequestBody {
    /// Optional expiration date of the invite
    pub expiration: Option<DateTime<Utc>>,
}

impl ExampleData for PostInviteRequestBody {
    fn example_data() -> Self {
        Self {
            expiration: Some(Utc.with_ymd_and_hms(2024, 6, 20, 14, 16, 19).unwrap()),
        }
    }
}
