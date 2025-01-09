// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use kustos::{
    policies_builder::{GrantingAccess, PoliciesBuilder},
    prelude::IsSubject,
    AccessMethod, Resource,
};
use opentalk_controller_service_facade::RequestUser;
use opentalk_controller_utils::CaptureApiError;
use opentalk_db_storage::{
    events::Event,
    rooms::{NewRoom, Room, UpdateRoom},
    sip_configs::NewSipConfig,
    utils::build_event_info,
};
use opentalk_types_api_v1::{
    error::ApiError,
    rooms::{by_room_id::GetRoomEventResponseBody, GetRoomsResponseBody, RoomResource},
};
use opentalk_types_common::{
    features,
    rooms::{RoomId, RoomPassword},
    tariffs::TariffResource,
    users::UserId,
};

use crate::{require_feature, ControllerBackend, ToUserProfile};

impl ControllerBackend {
    pub(super) async fn get_rooms(
        &self,
        current_user_id: UserId,
        per_page: i64,
        page: i64,
    ) -> Result<(GetRoomsResponseBody, i64), CaptureApiError> {
        let settings = self.settings.load();
        let mut conn = self.db.get_conn().await?;

        let accessible_rooms: kustos::AccessibleResources<RoomId> = self
            .authz
            .get_accessible_resources_for_user(current_user_id, AccessMethod::Get)
            .await?;

        let (rooms, room_count) = match accessible_rooms {
            kustos::AccessibleResources::All => {
                Room::get_all_with_creator_paginated(&mut conn, per_page, page).await?
            }
            kustos::AccessibleResources::List(list) => {
                Room::get_by_ids_with_creator_paginated(&mut conn, &list, per_page, page).await?
            }
        };

        let rooms = rooms
            .into_iter()
            .map(|(room, user)| RoomResource {
                id: room.id,
                created_by: user.to_public_user_profile(&settings),
                created_at: room.created_at.into(),
                password: room.password,
                waiting_room: room.waiting_room,
            })
            .collect::<Vec<RoomResource>>();

        Ok((GetRoomsResponseBody(rooms), room_count))
    }

    pub(super) async fn create_room(
        &self,
        current_user: RequestUser,
        password: Option<RoomPassword>,
        enable_sip: bool,
        waiting_room: bool,
        e2e_encryption: bool,
    ) -> Result<RoomResource, ApiError> {
        Ok(self
            .create_room_inner(
                current_user,
                password,
                enable_sip,
                waiting_room,
                e2e_encryption,
            )
            .await?)
    }

    async fn create_room_inner(
        &self,
        current_user: RequestUser,
        password: Option<RoomPassword>,
        enable_sip: bool,
        waiting_room: bool,
        e2e_encryption: bool,
    ) -> Result<RoomResource, CaptureApiError> {
        let settings = self.settings.load();
        let mut conn = self.db.get_conn().await?;

        if enable_sip {
            require_feature(&mut conn, &settings, current_user.id, &features::call_in()).await?;
        }

        let new_room = NewRoom {
            created_by: current_user.id,
            password,
            waiting_room,
            e2e_encryption,
            tenant_id: current_user.tenant_id,
        };

        let room = new_room.insert(&mut conn).await?;

        if enable_sip {
            _ = NewSipConfig::new(room.id, false).insert(&mut conn).await?;
        }

        drop(conn);

        let room_resource = RoomResource {
            id: room.id,
            created_by: current_user.to_public_user_profile(&settings),
            created_at: room.created_at.into(),
            password: room.password,
            waiting_room: room.waiting_room,
        };

        let policies = PoliciesBuilder::new()
            .grant_user_access(current_user.id)
            .room_read_access(room_resource.id)
            .room_write_access(room_resource.id)
            .finish();

        self.authz.add_policies(policies).await?;

        Ok(room_resource)
    }

    pub(super) async fn patch_room(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        password: Option<Option<RoomPassword>>,
        waiting_room: Option<bool>,
        e2e_encryption: Option<bool>,
    ) -> Result<RoomResource, ApiError> {
        Ok(self
            .patch_room_inner(
                current_user,
                room_id,
                password,
                waiting_room,
                e2e_encryption,
            )
            .await?)
    }

    async fn patch_room_inner(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        password: Option<Option<RoomPassword>>,
        waiting_room: Option<bool>,
        e2e_encryption: Option<bool>,
    ) -> Result<RoomResource, CaptureApiError> {
        let settings = self.settings.load();
        let mut conn = self.db.get_conn().await?;

        let changeset = UpdateRoom {
            password,
            waiting_room,
            e2e_encryption,
        };

        let room = changeset.apply(&mut conn, room_id).await?;

        let room_resource = RoomResource {
            id: room.id,
            created_by: current_user.to_public_user_profile(&settings),
            created_at: room.created_at.into(),
            password: room.password,
            waiting_room: room.waiting_room,
        };

        Ok(room_resource)
    }

    pub(super) async fn get_room(&self, room_id: &RoomId) -> Result<RoomResource, CaptureApiError> {
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

    pub(super) async fn get_room_tariff(
        &self,
        room_id: &RoomId,
    ) -> Result<TariffResource, CaptureApiError> {
        let settings = self.settings.load();
        let mut conn = self.db.get_conn().await?;

        let room = Room::get(&mut conn, *room_id).await?;
        let tariff = room.get_tariff(&mut conn).await?;

        let response = tariff.to_tariff_resource(
            settings.defaults.disabled_features.clone(),
            self.module_features.clone(),
        );

        Ok(response)
    }

    pub(super) async fn get_room_event(
        &self,
        room_id: &RoomId,
    ) -> Result<GetRoomEventResponseBody, CaptureApiError> {
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
            None => Err(ApiError::not_found().into()),
        }
    }
}

/// Provides functionality to grant room privileges
pub trait RoomsPoliciesBuilderExt {
    /// Set the room privileges needed to grant read access to guests
    #[allow(unused)]
    fn room_guest_read_access(self, room_id: RoomId) -> Self;
    /// Set the room privileges needed to grant read access
    fn room_read_access(self, room_id: RoomId) -> Self;
    /// Set the room privileges needed to grant write access
    fn room_write_access(self, room_id: RoomId) -> Self;
}

impl<T> RoomsPoliciesBuilderExt for PoliciesBuilder<GrantingAccess<T>>
where
    T: IsSubject + Clone,
{
    fn room_guest_read_access(self, room_id: RoomId) -> Self {
        self.add_resource(
            room_id.resource_id().with_suffix("/tariff"),
            [AccessMethod::Get],
        )
        .add_resource(
            room_id.resource_id().with_suffix("/event"),
            [AccessMethod::Get],
        )
    }

    fn room_read_access(self, room_id: RoomId) -> Self {
        self.add_resource(room_id.resource_id(), [AccessMethod::Get])
            .add_resource(
                room_id.resource_id().with_suffix("/invites"),
                [AccessMethod::Get],
            )
            .add_resource(
                room_id.resource_id().with_suffix("/streaming_targets"),
                [AccessMethod::Get],
            )
            .add_resource(
                room_id.resource_id().with_suffix("/start"),
                [AccessMethod::Post],
            )
            .add_resource(
                room_id.resource_id().with_suffix("/tariff"),
                [AccessMethod::Get],
            )
            .add_resource(
                room_id.resource_id().with_suffix("/event"),
                [AccessMethod::Get],
            )
            .add_resource(
                room_id.resource_id().with_suffix("/assets"),
                [AccessMethod::Get],
            )
            .add_resource(
                room_id.resource_id().with_suffix("/assets/*"),
                [AccessMethod::Get],
            )
    }

    fn room_write_access(self, room_id: RoomId) -> Self {
        self.add_resource(
            room_id.resource_id(),
            [AccessMethod::Patch, AccessMethod::Delete],
        )
        .add_resource(
            room_id.resource_id().with_suffix("/invites"),
            [AccessMethod::Post],
        )
        .add_resource(
            room_id.resource_id().with_suffix("/streaming_targets"),
            [AccessMethod::Post],
        )
        .add_resource(
            room_id.resource_id().with_suffix("/invites/*"),
            [AccessMethod::Get, AccessMethod::Put, AccessMethod::Delete],
        )
        .add_resource(
            room_id.resource_id().with_suffix("/streaming_targets/*"),
            [AccessMethod::Get, AccessMethod::Patch, AccessMethod::Delete],
        )
        .add_resource(
            room_id.resource_id().with_suffix("/assets"),
            [AccessMethod::Delete],
        )
        .add_resource(
            room_id.resource_id().with_suffix("/assets/*"),
            [AccessMethod::Delete],
        )
    }
}
