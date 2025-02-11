// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_types_common::time::Timestamp;
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_training_participation_report::TimeRange;

use super::{Checkpoint, TrainingReportState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RoomState {
    pub start: Timestamp,
    pub report_state: TrainingReportState,
    pub initial_checkpoint_delay: TimeRange,
    pub checkpoint_interval: TimeRange,
    pub history: Vec<Checkpoint>,
    pub next_checkpoint: Option<Timestamp>,
    pub known_participants: BTreeSet<ParticipantId>,
}
