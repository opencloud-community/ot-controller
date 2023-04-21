// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `recording` namespace

use crate::imports::*;

/// Data for the `set_consent` recording command
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SetConsent {
    /// Flag indicating whether the participant consents to being recorded
    pub consent: bool,
}
