// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Handles event favorites

use opentalk_controller_service_facade::RequestUser;
use opentalk_controller_utils::CaptureApiError;
use opentalk_db_storage::events::{Event, EventFavorite, NewEventFavorite};
use opentalk_types_api_v1::error::ApiError;
use opentalk_types_common::events::EventId;

use crate::ControllerBackend;

impl ControllerBackend {
    pub(crate) async fn add_event_to_favorites(
        &self,
        current_user: RequestUser,
        event_id: EventId,
    ) -> Result<bool, CaptureApiError> {
        let mut conn = self.db.get_conn().await?;

        let _event = Event::get(&mut conn, event_id).await?;

        let result = NewEventFavorite {
            user_id: current_user.id,
            event_id,
        }
        .try_insert(&mut conn)
        .await?;

        match result {
            Some(_) => Ok(true), // created
            None => Ok(false),   // no change
        }
    }

    pub(crate) async fn remove_event_from_favorites(
        &self,
        current_user: RequestUser,
        event_id: EventId,
    ) -> Result<(), CaptureApiError> {
        let mut conn = self.db.get_conn().await?;

        let existed = EventFavorite::delete_by_id(&mut conn, current_user.id, event_id).await?;

        if existed {
            Ok(())
        } else {
            Err(ApiError::not_found().into())
        }
    }
}
