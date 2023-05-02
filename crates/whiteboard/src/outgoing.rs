// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Serialize;
use types::signaling::whiteboard::event::{Error, PdfAsset};
use url::Url;

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "message")]
pub enum WhiteboardEvent {
    SpaceUrl(AccessUrl),
    PdfAsset(PdfAsset),
    Error(Error),
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct AccessUrl {
    pub url: Url,
}
