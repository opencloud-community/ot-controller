// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_controller_utils::CaptureApiError;
use opentalk_db_storage::rooms::Room;
use opentalk_signaling_core::{assets::verify_storage_usage, Participant};
use opentalk_types_api_v1::{
    error::ApiError,
    services::{recording::PostRecordingStartRequestBody, PostServiceStartResponseBody},
};

use crate::{signaling::ticket::start_or_continue_signaling_session, ControllerBackend};

impl ControllerBackend {
    pub(crate) async fn start_recording(
        &self,
        body: PostRecordingStartRequestBody,
    ) -> Result<PostServiceStartResponseBody, CaptureApiError> {
        let settings = self.settings_provider.get();
        let mut conn = self.db.get_conn().await?;
        let mut volatile = self.volatile.clone();

        if settings.rabbit_mq.recording_task_queue.is_none() {
            return Err(ApiError::not_found().into());
        }

        let room = Room::get(&mut conn, body.room_id).await?;

        verify_storage_usage(&mut conn, room.created_by).await?;

        let (ticket, resumption) = start_or_continue_signaling_session(
            &mut volatile,
            Participant::Recorder,
            room.id,
            body.breakout_room,
            None,
        )
        .await?;

        Ok(PostServiceStartResponseBody { ticket, resumption })
    }
}
