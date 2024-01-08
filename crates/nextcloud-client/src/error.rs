// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use reqwest::StatusCode;

use crate::ShareId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),

    #[error(transparent)]
    ReqwestDav(#[from] reqwest_dav::Error),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error("Server returned unauthorized error")]
    Unauthorized,

    #[error("Unknown share type")]
    UnknownShareType,

    #[error("Public upload was disabled by the admin")]
    PublicUploadDisabledByAdmin,

    #[error("File could not be shared")]
    FileCouldNotBeShared,

    #[error("Server sent unexpected status code {status_code}")]
    UnexpectedStatusCode { status_code: StatusCode },

    #[error("Wrong or no update parameter given")]
    WrongParameter,

    #[error("Share {share_id} not found")]
    ShareNotFound { share_id: ShareId },

    #[error("File {file_path} not found")]
    FileNotFound {
        file_path: String,
        #[source]
        source: reqwest_dav::Error,
    },
}
