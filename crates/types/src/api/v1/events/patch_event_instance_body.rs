// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{api::v1::events::EventStatus, core::DateTimeTz};

#[allow(unused_imports)]
use crate::imports::*;

/// Request body for the `PATCH /events/{event_id}/{instance_id}` endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, Validate))]
pub struct PatchEventInstanceBody {
    /// The title of th event
    #[cfg_attr(feature = "serde", validate(length(max = 255)))]
    pub title: Option<String>,
    /// The description of the event
    #[cfg_attr(feature = "serde", validate(length(max = 4096)))]
    pub description: Option<String>,
    /// Flag to indicate if the event is all-day
    pub is_all_day: Option<bool>,
    /// Start time of the event.
    pub starts_at: Option<DateTimeTz>,
    /// End time of the event.
    pub ends_at: Option<DateTimeTz>,
    /// Status of the event
    pub status: Option<EventStatus>,
}

impl PatchEventInstanceBody {
    /// Check if the body is empty
    pub fn is_empty(&self) -> bool {
        let PatchEventInstanceBody {
            title,
            description,
            is_all_day,
            starts_at,
            ends_at,
            status,
        } = self;

        title.is_none()
            && description.is_none()
            && is_all_day.is_none()
            && starts_at.is_none()
            && ends_at.is_none()
            && status.is_none()
    }
}
