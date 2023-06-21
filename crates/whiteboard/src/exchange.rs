// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};
use types::signaling::whiteboard::event::PdfAsset;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    /// Spacedeck has been initialized
    Initialized,
    PdfAsset(PdfAsset),
}
