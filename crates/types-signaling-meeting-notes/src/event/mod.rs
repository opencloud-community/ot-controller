// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to signaling events in the `meeting-notes` namespace

mod access_url;
mod error;
mod pdf_asset;

pub use access_url::AccessUrl;
pub use error::Error;
pub use pdf_asset::PdfAsset;
