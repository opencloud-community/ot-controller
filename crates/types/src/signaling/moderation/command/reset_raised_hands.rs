// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_types_signaling::ParticipantId;

#[allow(unused_imports)]
use crate::imports::*;

/// Reset raised hands for the meeting
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ResetRaisedHands {
    /// An optional single participant to reset the raised hand for
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            with = "opentalk_types_common::collections::one_or_many_btree_set_option"
        )
    )]
    pub target: Option<BTreeSet<ParticipantId>>,
}
