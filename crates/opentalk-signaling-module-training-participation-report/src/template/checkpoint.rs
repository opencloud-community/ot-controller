// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use chrono_tz::Tz;
use opentalk_report_generation::{ReportDateTime, ToReportDateTime};
use opentalk_types_signaling::ParticipantId;

use crate::storage;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct Checkpoint {
    pub timestamp: ReportDateTime,
    pub presence: BTreeMap<ParticipantId, ReportDateTime>,
}

impl Checkpoint {
    pub fn from_storage_checkpoint(
        storage::Checkpoint {
            timestamp,
            presence,
        }: &storage::Checkpoint,
        report_tz: &Tz,
    ) -> Self {
        Self {
            timestamp: timestamp.to_report_date_time(report_tz),
            presence: presence
                .iter()
                .map(|(participant, timestamp)| {
                    (*participant, timestamp.to_report_date_time(report_tz))
                })
                .collect(),
        }
    }
}
