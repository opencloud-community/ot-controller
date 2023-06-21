// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum WhiteboardCommand {
    /// Initialize a new space for the room
    ///
    /// There can only be one space per room
    Initialize,
    /// Generates the current whiteboard as PDF.
    GeneratePdf,
}
