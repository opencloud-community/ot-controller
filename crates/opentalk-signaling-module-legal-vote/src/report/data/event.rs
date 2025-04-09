// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use super::{MaybeUserName, ResolvedReportedIssue};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "event_details")]
pub enum Event {
    UserJoined(MaybeUserName),
    UserLeft(MaybeUserName),
    Issue(ResolvedReportedIssue),
}
