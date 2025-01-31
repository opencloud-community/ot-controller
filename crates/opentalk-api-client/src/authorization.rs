// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::rooms::invite_codes::InviteCode;

#[derive(Debug)]
pub struct InviteCodeAuthorization(InviteCode);

impl From<InviteCode> for InviteCodeAuthorization {
    fn from(value: InviteCode) -> Self {
        Self(value)
    }
}

impl opentalk_client_shared::Authorization for InviteCodeAuthorization {
    fn apply_authorization_headers(&self, headers: &mut http::HeaderMap) {
        let _ = headers.insert(
            http::header::AUTHORIZATION,
            http::HeaderValue::from_str(&format!("InviteCode {}", self.0))
                .expect("valid header value"),
        );
    }
}
