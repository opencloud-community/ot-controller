// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Provides room-related implementation

use std::str::FromStr;

use kustos::{
    policies_builder::{GrantingAccess, PoliciesBuilder},
    prelude::IsSubject,
    AccessMethod, Resource,
};
use opentalk_controller_service_facade::RequestUser;
use opentalk_controller_utils::{
    deletion::{Deleter, RoomDeleter},
    CaptureApiError,
};
use opentalk_db_storage::{
    events::Event,
    invites::Invite,
    rooms::{NewRoom, Room, UpdateRoom},
    sip_configs::NewSipConfig,
    tariffs::Tariff,
    utils::build_event_info,
};
use opentalk_signaling_core::Participant;
use opentalk_types_api_v1::{
    error::{ApiError, ValidationErrorEntry, ERROR_CODE_INVALID_VALUE},
    pagination::PagePaginationQuery,
    rooms::{
        by_room_id::{
            GetRoomEventResponseBody, PostRoomsStartInvitedRequestBody, PostRoomsStartRequestBody,
            RoomsStartResponseBody,
        },
        GetRoomsResponseBody, RoomResource,
    },
};
use opentalk_types_common::{
    features,
    rooms::{invite_codes::InviteCode, RoomId, RoomPassword},
    tariffs::TariffResource,
    users::UserId,
};

use crate::{
    controller_backend::rooms::start_room_error::StartRoomError,
    require_feature,
    signaling::{
        ticket::start_or_continue_signaling_session,
        ws_modules::{breakout::BreakoutStorageProvider, moderation::ModerationStorageProvider},
    },
    ControllerBackend, ToUserProfile,
};

pub mod start_room_error;

impl ControllerBackend {
    pub(crate) async fn get_rooms(
        &self,
        current_user_id: UserId,
        pagination: &PagePaginationQuery,
    ) -> Result<(GetRoomsResponseBody, i64), CaptureApiError> {
        let settings = self.settings_provider.get();
        let mut conn = self.db.get_conn().await?;

        let accessible_rooms: kustos::AccessibleResources<RoomId> = self
            .authz
            .get_accessible_resources_for_user(current_user_id, AccessMethod::Get)
            .await?;

        let (rooms, room_count) = match accessible_rooms {
            kustos::AccessibleResources::All => {
                Room::get_all_with_creator_paginated(
                    &mut conn,
                    pagination.per_page,
                    pagination.page,
                )
                .await?
            }
            kustos::AccessibleResources::List(list) => {
                Room::get_by_ids_with_creator_paginated(
                    &mut conn,
                    &list,
                    pagination.per_page,
                    pagination.page,
                )
                .await?
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

    pub(crate) async fn create_room(
        &self,
        current_user: RequestUser,
        password: Option<RoomPassword>,
        enable_sip: bool,
        waiting_room: bool,
        e2e_encryption: bool,
    ) -> Result<RoomResource, CaptureApiError> {
        let settings = self.settings_provider.get();
        let mut conn = self.db.get_conn().await?;

        if enable_sip {
            require_feature(
                &mut conn,
                &settings,
                current_user.id,
                &features::CALL_IN_MODULE_FEATURE_ID,
            )
            .await?;
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

    pub(crate) async fn patch_room(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        password: Option<Option<RoomPassword>>,
        waiting_room: Option<bool>,
        e2e_encryption: Option<bool>,
    ) -> Result<RoomResource, CaptureApiError> {
        let settings = self.settings_provider.get();
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

    pub(crate) async fn delete_room(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        force_delete_reference_if_external_services_fail: bool,
    ) -> Result<(), CaptureApiError> {
        let settings = self.settings_provider.get();
        let mut conn = self.db.get_conn().await?;

        let deleter = RoomDeleter::new(room_id, force_delete_reference_if_external_services_fail);

        deleter
            .perform(
                log::logger(),
                &mut conn,
                &self.authz,
                Some(current_user.id),
                self.exchange_handle.clone(),
                &settings,
                &self.storage,
            )
            .await?;

        Ok(())
    }

    pub(crate) async fn get_room(&self, room_id: &RoomId) -> Result<RoomResource, CaptureApiError> {
        let settings = self.settings_provider.get();
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

    pub(crate) async fn get_room_tariff(
        &self,
        room_id: &RoomId,
    ) -> Result<TariffResource, CaptureApiError> {
        let settings = self.settings_provider.get();
        let mut conn = self.db.get_conn().await?;

        let room = Room::get(&mut conn, *room_id).await?;
        let tariff = room.get_tariff(&mut conn).await?;

        let response = tariff.to_tariff_resource(
            settings.raw.defaults.disabled_features.clone(),
            self.module_features.clone(),
        );

        Ok(response)
    }

    pub(crate) async fn get_room_event(
        &self,
        room_id: &RoomId,
    ) -> Result<GetRoomEventResponseBody, CaptureApiError> {
        let settings = self.settings_provider.get();
        let mut conn = self.db.get_conn().await?;

        let event = Event::get_for_room(&mut conn, *room_id).await?;

        let room = Room::get(&mut conn, *room_id).await?;

        let tariff = Tariff::get_by_user_id(&mut conn, &room.created_by).await?;

        match event.as_ref() {
            Some(event) => {
                let call_in_tel = settings.call_in.as_ref().map(|call_in| call_in.tel.clone());
                let event_info = build_event_info(
                    &mut conn,
                    call_in_tel,
                    *room_id,
                    room.e2e_encryption,
                    event,
                    &tariff,
                )
                .await?;
                Ok(GetRoomEventResponseBody(event_info))
            }
            None => Err(ApiError::not_found().into()),
        }
    }

    pub(crate) async fn start_room_session(
        &self,
        current_user: RequestUser,
        room_id: RoomId,
        request: PostRoomsStartRequestBody,
    ) -> Result<RoomsStartResponseBody, CaptureApiError> {
        let mut conn = self.db.get_conn().await?;
        let mut volatile = self.volatile.clone();

        let room = Room::get(&mut conn, room_id).await?;

        // check if user is banned from room
        if volatile
            .moderation_storage()
            .is_user_banned(room.id, current_user.id)
            .await
            .map_err(Into::<ApiError>::into)?
        {
            return Err(StartRoomError::BannedFromRoom.into());
        }

        if let Some(breakout_room) = request.breakout_room {
            let config = volatile
                .breakout_storage()
                .get_breakout_config(room.id)
                .await
                .map_err(Into::<ApiError>::into)?;

            if let Some(config) = config {
                if !config.is_valid_id(breakout_room) {
                    return Err(StartRoomError::InvalidBreakoutRoomId.into());
                }
            } else {
                return Err(StartRoomError::NoBreakoutRooms.into());
            }
        }

        let (ticket, resumption) = start_or_continue_signaling_session(
            &mut volatile,
            current_user.id.into(),
            room_id,
            request.breakout_room,
            request.resumption,
        )
        .await?;

        Ok(RoomsStartResponseBody { ticket, resumption })
    }

    pub(crate) async fn start_invited_room_session(
        &self,
        room_id: RoomId,
        request: PostRoomsStartInvitedRequestBody,
    ) -> Result<RoomsStartResponseBody, CaptureApiError> {
        let mut volatile = self.volatile.clone();

        let invite_code_as_uuid = uuid::Uuid::from_str(&request.invite_code).map_err(|_| {
            ApiError::unprocessable_entities([ValidationErrorEntry::new(
                "invite_code",
                ERROR_CODE_INVALID_VALUE,
                Some("Bad invite code format"),
            )])
        })?;

        let mut conn = self.db.get_conn().await?;

        let invite = Invite::get(&mut conn, InviteCode::from(invite_code_as_uuid)).await?;

        if !invite.active {
            return Err(ApiError::not_found().into());
        }

        if invite.room != room_id {
            return Err(ApiError::bad_request()
                .with_message("Room id mismatch")
                .into());
        }

        let room = Room::get(&mut conn, invite.room).await?;

        drop(conn);

        if let Some(password) = &room.password {
            if let Some(pw) = &request.password {
                if pw != password {
                    return Err(StartRoomError::WrongRoomPassword.into());
                }
            } else {
                return Err(StartRoomError::WrongRoomPassword.into());
            }
        }

        if let Some(breakout_room) = request.breakout_room {
            let config = volatile
                .breakout_storage()
                .get_breakout_config(room.id)
                .await
                .map_err(Into::<ApiError>::into)?;

            if let Some(config) = config {
                if !config.is_valid_id(breakout_room) {
                    return Err(StartRoomError::InvalidBreakoutRoomId.into());
                }
            } else {
                return Err(StartRoomError::NoBreakoutRooms.into());
            }
        }

        let (ticket, resumption) = start_or_continue_signaling_session(
            &mut volatile,
            Participant::Guest,
            room_id,
            request.breakout_room,
            request.resumption,
        )
        .await?;

        Ok(RoomsStartResponseBody { ticket, resumption })
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
            [AccessMethod::Post, AccessMethod::Delete],
        )
        .add_resource(
            room_id.resource_id().with_suffix("/assets/*"),
            [AccessMethod::Delete],
        )
    }
}
