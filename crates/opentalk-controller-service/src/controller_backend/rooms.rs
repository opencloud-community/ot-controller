// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage::{events::Event, rooms::Room, utils::build_event_info};
use opentalk_types::api::error::ApiError;
use opentalk_types_api_v1::rooms::{by_room_id::GetRoomEventResponseBody, RoomResource};
use opentalk_types_common::rooms::RoomId;

use crate::{ControllerBackend, ToUserProfile};

impl ControllerBackend {
    pub(super) async fn get_room(&self, room_id: &RoomId) -> Result<RoomResource, ApiError> {
        let settings = self.settings.load();

        let mut conn = self.db.get_conn().await?;

        let (room, created_by) = Room::get_with_user(&mut conn, *room_id).await?;

        let room_resource = RoomResource {
            id: room.id,
            created_by: created_by.to_public_user_profile(&settings),
            created_at: room.created_at.into(),
            password: room.password,
            waiting_room: room.waiting_room,
        };

        Ok(room_resource)
    }

    pub(super) async fn get_room_event(
        &self,
        room_id: &RoomId,
    ) -> Result<GetRoomEventResponseBody, ApiError> {
        let settings = self.settings.load();

        let mut conn = self.db.get_conn().await?;

        let event = Event::get_for_room(&mut conn, *room_id).await?;

        let room = Room::get(&mut conn, *room_id).await?;

        match event.as_ref() {
            Some(event) => {
                let call_in_tel = settings.call_in.as_ref().map(|call_in| call_in.tel.clone());
                let event_info =
                    build_event_info(&mut conn, call_in_tel, *room_id, room.e2e_encryption, event)
                        .await?;
                Ok(GetRoomEventResponseBody(event_info))
            }
            None => Err(ApiError::not_found()),
        }
    }
}
