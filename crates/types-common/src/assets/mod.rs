// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Data types for handling assets.

mod asset_id;
mod file_extension;

pub use asset_id::AssetId;
pub use file_extension::{FileExtension, MAX_FILE_EXTENSION_LENGTH};
