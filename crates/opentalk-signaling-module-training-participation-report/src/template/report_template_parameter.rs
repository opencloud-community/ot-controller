// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use chrono_tz::Tz;
use opentalk_report_generation::{ReportDateTime, ToReportDateTime};
use opentalk_types_common::{
    events::{EventDescription, EventTitle},
    time::Timestamp,
    users::DisplayName,
};
use opentalk_types_signaling::ParticipantId;

use super::Checkpoint;
use crate::storage::RoomState;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ReportTemplateParameter {
    pub title: EventTitle,
    pub description: EventDescription,
    pub start: ReportDateTime,
    pub end: ReportDateTime,
    pub report_timezone: Tz,
    pub participants: BTreeMap<ParticipantId, Option<DisplayName>>,
    pub checkpoints: Vec<Checkpoint>,
}

impl ReportTemplateParameter {
    pub(crate) fn build(
        room_state: &RoomState,
        report_tz: &Tz,
        participants: BTreeMap<ParticipantId, Option<DisplayName>>,
        title: EventTitle,
        description: EventDescription,
        end: Timestamp,
    ) -> Self {
        let checkpoints = room_state
            .history
            .iter()
            .map(|storage_checkpoint| {
                Checkpoint::from_storage_checkpoint(storage_checkpoint, report_tz)
            })
            .collect();
        Self {
            title,
            description,
            start: room_state.start.to_report_date_time(report_tz),
            end: end.to_report_date_time(report_tz),
            report_timezone: *report_tz,
            participants,
            checkpoints,
        }
    }
}
