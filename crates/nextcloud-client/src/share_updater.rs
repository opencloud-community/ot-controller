// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashSet;

use chrono::NaiveDate;
use log::warn;
use reqwest::StatusCode;
use serde::Serialize;

use crate::{
    types::{OcsShareAnswer, OcsShareData, ShareAnswer},
    Client, Error, Result, ShareId, SharePermission,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
enum ParameterUpdate {
    PublicUpload(bool),
    Permissions(#[serde(with = "crate::utils::share_permissions")] HashSet<SharePermission>),
    ExpireDate(String),
    Note(String),
    Label(String),
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct Parameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    public_upload: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
    #[serde(
        with = "crate::utils::optional_share_permissions",
        skip_serializing_if = "Option::is_none"
    )]
    permissions: Option<HashSet<SharePermission>>,
    #[serde(
        with = "crate::utils::optional_naive_date",
        skip_serializing_if = "Option::is_none"
    )]
    expire_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
}

#[must_use]
pub struct ShareUpdater {
    client: Client,
    share_id: ShareId,
}

impl ShareUpdater {
    pub(crate) fn new(client: Client, share_id: ShareId) -> Self {
        Self { client, share_id }
    }

    pub async fn public_upload(self, public_upload: bool) -> Result<OcsShareAnswer<OcsShareData>> {
        self.send(ParameterUpdate::PublicUpload(public_upload))
            .await
    }

    pub async fn permissions(
        self,
        permissions: HashSet<SharePermission>,
    ) -> Result<OcsShareAnswer<OcsShareData>> {
        self.send(ParameterUpdate::Permissions(permissions)).await
    }

    pub async fn expire_date(
        self,
        expire_date: Option<NaiveDate>,
    ) -> Result<OcsShareAnswer<OcsShareData>> {
        self.send(ParameterUpdate::ExpireDate(
            expire_date
                .map(|date| date.format("%Y-%m-%d").to_string())
                .unwrap_or_default(),
        ))
        .await
    }

    pub async fn note<N: Into<String>>(self, note: N) -> Result<OcsShareAnswer<OcsShareData>> {
        self.send(ParameterUpdate::Note(note.into())).await
    }

    pub async fn label<L: Into<String>>(self, label: L) -> Result<OcsShareAnswer<OcsShareData>> {
        self.send(ParameterUpdate::Label(label.into())).await
    }

    async fn send(self, parameter: ParameterUpdate) -> Result<OcsShareAnswer<OcsShareData>> {
        let Self { client, share_id } = self;

        let url = client
            .share_api_base_url()?
            .join("shares/")?
            .join(share_id.as_str())?;
        let request = client
            .inner
            .http_client
            .put(url)
            .basic_auth(&client.inner.username, Some(&client.inner.password))
            .json(&parameter);
        let answer = request.send().await?;

        match answer.status() {
            StatusCode::CONTINUE | StatusCode::OK => {}
            StatusCode::BAD_REQUEST => {
                // 400
                return Err(Error::WrongParameter);
            }
            StatusCode::UNAUTHORIZED => {
                // 401
                return Err(Error::Unauthorized);
            }
            StatusCode::FORBIDDEN => {
                // 403
                return Err(Error::PublicUploadDisabledByAdmin);
            }
            StatusCode::NOT_FOUND => {
                // 404
                return Err(Error::ShareNotFound { share_id });
            }
            status_code => {
                warn!("Received unexpected status code {status_code} from NextCloud server.");
                match answer.text().await {
                    Ok(text) => {
                        warn!("Response for unexpected status code {status_code}:\n{text}");
                    }
                    Err(e) => {
                        warn!("Error retrieving body from NextCloud: {}", e);
                    }
                }
                return Err(Error::UnexpectedStatusCode { status_code });
            }
        }
        let answer: ShareAnswer<OcsShareData> = answer.json().await?;
        Ok(answer.ocs)
    }
}
