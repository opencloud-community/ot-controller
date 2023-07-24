// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `whiteboard` namespace

#[allow(unused_imports)]
use crate::imports::*;

/// Commands for the `whiteboard` namespace
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "action", rename_all = "snake_case")
)]
pub enum WhiteboardCommand {
    /// Initialize a new space for the room
    ///
    /// There can only be one space per room
    Initialize,

    /// Generates the current whiteboard as PDF.
    GeneratePdf,
}
