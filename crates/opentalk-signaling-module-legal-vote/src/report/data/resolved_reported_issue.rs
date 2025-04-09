// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::users::DisplayName;
use opentalk_types_signaling_legal_vote::issue::Issue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedReportedIssue {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<DisplayName>,

    #[serde(flatten)]
    pub issue: Issue,
}
