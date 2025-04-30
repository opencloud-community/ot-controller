// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::settings_file;

/// MinIO settings.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct MinIO {
    /// The URI of the S3 storage.
    pub uri: String,

    /// The bucket in the S3 storage.
    pub bucket: String,

    /// The access key to the storage.
    pub access_key: String,

    /// The secret key to the storage.
    pub secret_key: String,
}

impl From<settings_file::MinIO> for MinIO {
    fn from(
        settings_file::MinIO {
            uri,
            bucket,
            access_key,
            secret_key,
        }: settings_file::MinIO,
    ) -> Self {
        Self {
            uri,
            bucket,
            access_key,
            secret_key,
        }
    }
}
