// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_api_v1::auth::GetLoginResponseBody;

use crate::ControllerBackend;

impl ControllerBackend {
    pub(super) async fn get_login(&self) -> GetLoginResponseBody {
        GetLoginResponseBody {
            oidc: self.frontend_oidc_provider.clone(),
        }
    }
}
