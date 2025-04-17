// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_controller_utils::CaptureApiError;
use opentalk_db_storage::sip_configs::SipConfig;
use opentalk_signaling_core::Participant;
use opentalk_types_api_v1::{
    error::ApiError,
    services::{call_in::PostCallInStartRequestBody, PostServiceStartResponseBody},
};
use opentalk_types_common::features;

use crate::{
    require_feature, signaling::ticket::start_or_continue_signaling_session, ControllerBackend,
};

impl ControllerBackend {
    pub(crate) async fn start_call_in(
        &self,
        request: PostCallInStartRequestBody,
    ) -> Result<PostServiceStartResponseBody, CaptureApiError> {
        let settings = self.settings_provider.get_raw();
        let mut conn = self.db.get_conn().await?;
        let mut volatile = self.volatile.clone();

        let (sip_config, room) = SipConfig::get_with_room(&mut conn, &request.id)
            .await?
            .ok_or_else(invalid_credentials_error)?;

        if room.e2e_encryption {
            return Err(ApiError::forbidden()
                .with_code("service_unavailable")
                .with_message("call-in is not available for encrypted rooms")
                .into());
        }

        require_feature(
            &mut conn,
            &settings,
            room.created_by,
            &features::CALL_IN_MODULE_FEATURE_ID,
        )
        .await?;

        if sip_config.password != request.pin {
            return Err(invalid_credentials_error().into());
        }

        drop(conn);

        let (ticket, resumption) = start_or_continue_signaling_session(
            &mut volatile,
            Participant::Sip,
            room.id,
            None,
            None,
        )
        .await?;

        Ok(PostServiceStartResponseBody { ticket, resumption })
    }
}

pub fn invalid_credentials_error() -> ApiError {
    ApiError::bad_request()
        .with_code("invalid_credentials")
        .with_message("given call-in id & pin combination is not valid")
}
