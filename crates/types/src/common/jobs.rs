// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! A list of known types used by the job executor.

#[allow(unused_imports)]
use crate::imports::*;

/// Maintenance job types that can be executed by OpenTalk
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    strum::AsRefStr,
    strum::Display,
    strum::EnumCount,
    strum::EnumIter,
    strum::EnumString,
    strum::EnumVariantNames,
    strum::IntoStaticStr,
)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum JobType {
    /// A simple self-check of the job execution system
    SelfCheck,

    /// A job for cleaning up events that ended at minimum a defined duration ago
    EventCleanup,

    /// A job to cleanup adhoc events a certain duration after they were created
    AdhocEventCleanup,

    /// A job for cleaning up expired invites
    InviteCleanup,
}
