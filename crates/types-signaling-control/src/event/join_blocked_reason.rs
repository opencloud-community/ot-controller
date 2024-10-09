// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// The reason for blocking a participant from joining a meeting
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "reason", rename_all = "snake_case")
)]
pub enum JoinBlockedReason {
    /// The participant limit for the meeting's tariff has been reached
    ParticipantLimitReached,
}
