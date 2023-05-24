// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashSet;

use serde::Deserialize;

use crate::{ShareId, SharePermission};

#[derive(Debug, Deserialize)]
pub struct ShareAnswer {
    pub ocs: OcsShareAnswer,
}

#[derive(Debug, Deserialize)]
pub struct OcsShareAnswer {
    pub meta: Meta,
    pub data: OcsShareData,
}

#[derive(Debug, Deserialize)]
pub struct OcsShareData {
    pub id: ShareId,
    pub url: String,
    pub file_target: String,
    #[serde(with = "crate::utils::share_permissions")]
    pub permissions: HashSet<SharePermission>,
}

#[derive(Debug, Deserialize)]
pub struct Meta {
    pub message: String,
    pub status: String,
    pub statuscode: u32,
}
