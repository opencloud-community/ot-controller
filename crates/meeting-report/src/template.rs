// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types used inside the tera template.
//!
// IMPORTANT: when changing the structs below, make sure to update the following documentation:
// * docs/admin/core/meeting_reports.md

use chrono::{DateTime, Utc};
use opentalk_types_common::time::TimeZone;
use opentalk_types_signaling::{ParticipantId, ParticipationKind, Role};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ReportTemplateParameter {
    pub title: String,
    pub description: String,
    pub starts_at: Option<DateTime<Utc>>,
    pub starts_at_tz: Option<TimeZone>,
    pub ends_at: Option<DateTime<Utc>>,
    pub ends_at_tz: Option<TimeZone>,
    pub participants: Vec<ReportParticipant>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ReportParticipant {
    pub id: ParticipantId,
    pub name: String,
    pub role: Role,
    pub kind: ParticipationKind,
    pub email: Option<String>,
    pub joined_at: Option<DateTime<Utc>>,
    pub left_at: Option<DateTime<Utc>>,
}
