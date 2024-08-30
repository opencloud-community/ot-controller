// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_types_common::streaming::StreamingTargetId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action")]
#[allow(clippy::enum_variant_names)]
pub enum Message {
    StartStreams {
        target_ids: BTreeSet<StreamingTargetId>,
    },
    StopStreams {
        target_ids: BTreeSet<StreamingTargetId>,
    },
    PauseStreams {
        target_ids: BTreeSet<StreamingTargetId>,
    },
}
