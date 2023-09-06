// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashSet;
use std::fmt::Debug;

use serde::Deserialize;

use crate::{ShareId, SharePermission};

#[derive(Debug, Deserialize)]
pub struct ShareAnswer<D: Debug> {
    pub ocs: OcsShareAnswer<D>,
}

#[derive(Debug, Deserialize)]
pub struct OcsShareAnswer<D: Debug> {
    pub meta: Meta,
    pub data: D,
}

#[derive(Debug, Deserialize)]
pub struct Meta {
    pub message: String,
    pub status: String,
    pub statuscode: u32,
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
pub struct OcsPassword {
    pub password: String,
}
