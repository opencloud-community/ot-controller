// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Room related API structs and Endpoints
//!
//! The defined structs are exposed to the REST API and will be serialized/deserialized. Similar
//! structs are defined in the Database crate [`opentalk_db_storage`] for database operations.

use std::str::FromStr;

use super::response::NoContent;
use crate::{
    api::{
        signaling::{
            breakout, moderation, ticket::start_or_continue_signaling_session, SignalingModules,
        },
        v1::{util::require_feature, ApiResponse},
    },
    settings::SharedSettingsActix,
};

use actix_web::{
    delete, get, patch, post,
    web::{self, Data, Json, Path, ReqData},
};
use kustos::{
    policies_builder::{GrantingAccess, PoliciesBuilder},
    subject::IsSubject,
    AccessMethod, Authz, Resource, ResourceId,
};
use opentalk_database::Db;
use opentalk_db_storage::utils::build_event_info;
use opentalk_db_storage::{
    events::Event,
    invites::Invite,
    rooms::{self as db_rooms, Room},
    sip_configs::NewSipConfig,
    users::User,
};
use opentalk_signaling_core::{Participant, RedisConnection};
use opentalk_types::{
    api::{
        error::{ApiError, ValidationErrorEntry, ERROR_CODE_INVALID_VALUE},
        v1::{
            pagination::PagePaginationQuery,
            rooms::{
                GetRoomEventResponse, PatchRoomsRequestBody, PostRoomsRequestBody,
                PostRoomsStartInvitedRequestBody, PostRoomsStartRequestBody, RoomResource,
                RoomsStartResponse, StartRoomError,
            },
        },
    },
    common::{features, tariff::TariffResource},
    core::{InviteCodeId, RoomId},
};
use validator::Validate;

/// API Endpoint *GET /rooms*
///
/// Returns a JSON array of all accessible rooms as [`Room`]
#[get("/rooms")]
pub async fn accessible(
    settings: SharedSettingsActix,
    db: Data<Db>,
    current_user: ReqData<User>,
    pagination: web::Query<PagePaginationQuery>,
    authz: Data<Authz>,
) -> Result<ApiResponse<Vec<RoomResource>>, ApiError> {
    let settings = settings.load();
    let current_user = current_user.into_inner();
    let PagePaginationQuery { per_page, page } = pagination.into_inner();

    let accessible_rooms: kustos::AccessibleResources<RoomId> = authz
        .get_accessible_resources_for_user(current_user.id, AccessMethod::Get)
        .await?;

    let mut conn = db.get_conn().await?;

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
            created_at: room.created_at,
            password: room.password,
            waiting_room: room.waiting_room,
        })
        .collect::<Vec<RoomResource>>();

    Ok(ApiResponse::new(rooms).with_page_pagination(per_page, page, room_count))
}

/// API Endpoint *POST /rooms*
///
/// Uses the provided [`PostRoomsRequestBody`] to create a new room.
/// Returns the created [`RoomResource`].
#[post("/rooms")]
pub async fn new(
    settings: SharedSettingsActix,
    db: Data<Db>,
    authz: Data<Authz>,
    current_user: ReqData<User>,
    body: Json<PostRoomsRequestBody>,
) -> Result<Json<RoomResource>, ApiError> {
    let settings = settings.load();
    let current_user = current_user.into_inner();
    let room_parameters = body.into_inner();

    room_parameters.validate()?;

    let mut conn = db.get_conn().await?;

    if room_parameters.enable_sip {
        require_feature(&mut conn, &settings, current_user.id, features::CALL_IN).await?;
    }

    let new_room = db_rooms::NewRoom {
        created_by: current_user.id,
        password: room_parameters.password,
        waiting_room: room_parameters.waiting_room,
        tenant_id: current_user.tenant_id,
    };

    let room = new_room.insert(&mut conn).await?;

    if room_parameters.enable_sip {
        NewSipConfig::new(room.id, false).insert(&mut conn).await?;
    }

    drop(conn);

    let room_resource = RoomResource {
        id: room.id,
        created_by: current_user.to_public_user_profile(&settings),
        created_at: room.created_at,
        password: room.password,
        waiting_room: room.waiting_room,
    };

    let policies = PoliciesBuilder::new()
        .grant_user_access(current_user.id)
        .room_read_access(room_resource.id)
        .room_write_access(room_resource.id)
        .finish();

    authz.add_policies(policies).await?;

    Ok(Json(room_resource))
}

/// API Endpoint *PATCH /rooms/{room_id}*
///
/// Uses the provided [`PatchRoomsRequestBody`] to modify a specified room.
/// Returns the modified [`RoomResource`]
#[patch("/rooms/{room_id}")]
pub async fn patch(
    settings: SharedSettingsActix,
    db: Data<Db>,
    current_user: ReqData<User>,
    room_id: Path<RoomId>,
    body: Json<PatchRoomsRequestBody>,
) -> Result<Json<RoomResource>, ApiError> {
    let settings = settings.load();
    let current_user = current_user.into_inner();
    let room_id = room_id.into_inner();
    let modify_room = body.into_inner();

    modify_room.validate()?;

    let mut conn = db.get_conn().await?;

    let changeset = db_rooms::UpdateRoom {
        password: modify_room.password,
        waiting_room: modify_room.waiting_room,
    };

    let room = changeset.apply(&mut conn, room_id).await?;

    let room_resource = RoomResource {
        id: room.id,
        created_by: current_user.to_public_user_profile(&settings),
        created_at: room.created_at,
        password: room.password,
        waiting_room: room.waiting_room,
    };

    Ok(Json(room_resource))
}

/// API Endpoint *DELETE /rooms/{room_id}*
///
/// Deletes the room and owned resources.
#[delete("/rooms/{room_id}")]
pub async fn delete(
    db: Data<Db>,
    room_id: Path<RoomId>,
    authz: Data<Authz>,
) -> Result<NoContent, ApiError> {
    let room_id = room_id.into_inner();

    Room::delete_by_id(&mut db.get_conn().await?, room_id).await?;

    let resources = associated_resource_ids(room_id);

    authz.remove_explicit_resources(resources).await?;

    Ok(NoContent)
}

/// API Endpoint *GET /rooms/{room_id}*
///
/// Returns the specified Room as [`RoomResource`].
#[get("/rooms/{room_id}")]
pub async fn get(
    settings: SharedSettingsActix,
    db: Data<Db>,
    room_id: Path<RoomId>,
) -> Result<Json<RoomResource>, ApiError> {
    let settings = settings.load();
    let room_id = room_id.into_inner();

    let mut conn = db.get_conn().await?;

    let (room, created_by) = Room::get_with_user(&mut conn, room_id).await?;

    let room_resource = RoomResource {
        id: room.id,
        created_by: created_by.to_public_user_profile(&settings),
        created_at: room.created_at,
        password: room.password,
        waiting_room: room.waiting_room,
    };

    Ok(Json(room_resource))
}

#[get("/rooms/{room_id}/tariff")]
pub async fn get_room_tariff(
    shared_settings: SharedSettingsActix,
    db: Data<Db>,
    modules: Data<SignalingModules>,
    room_id: Path<RoomId>,
) -> Result<Json<TariffResource>, ApiError> {
    let settings = shared_settings.load_full();

    let room_id = room_id.into_inner();

    let mut conn = db.get_conn().await?;

    let room = Room::get(&mut conn, room_id).await?;
    let tariff = room.get_tariff(&mut conn).await?;

    let response = tariff.to_tariff_resource(
        settings.defaults.disabled_features(),
        modules.get_module_features(),
    );

    Ok(Json(response))
}

#[get("/rooms/{room_id}/event")]
pub async fn get_room_event(
    settings: SharedSettingsActix,
    db: Data<Db>,
    room_id: Path<RoomId>,
) -> Result<Json<GetRoomEventResponse>, ApiError> {
    let settings = settings.load();
    let room_id = room_id.into_inner();

    let mut conn = db.get_conn().await?;

    let event = Event::get_for_room(&mut conn, room_id).await?;

    match event.as_ref() {
        Some(event) => {
            let call_in_tel = settings.call_in.as_ref().map(|call_in| call_in.tel.clone());
            let event_info = build_event_info(&mut conn, call_in_tel, room_id, event).await?;
            Ok(Json(GetRoomEventResponse(event_info)))
        }
        None => Err(ApiError::not_found()),
    }
}

/// API Endpoint *POST /rooms/{room_id}/start*
///
/// This endpoint has to be called in order to get a room ticket. When joining a room, the ticket
/// must be provided as a `Sec-WebSocket-Protocol` header field when starting the WebSocket
/// connection.
///
/// When the requested room has a password set, the requester has to provide the correct password
/// through the [`PostRoomsStartRequestBody`] JSON in the requests body. When the room has no password set,
/// the provided password will be ignored.
///
/// Returns a [`RoomsStartResponse`] containing the ticket for the specified room.
///
/// # Errors
///
/// Returns [`StartRoomError::WrongRoomPassword`] when the provided password is wrong
/// Returns [`StartRoomError::NoBreakoutRooms`]  when no breakout rooms are configured but were provided
/// Returns [`StartRoomError::InvalidBreakoutRoomId`]  when the provided breakout room id is invalid     
#[post("/rooms/{room_id}/start")]
pub async fn start(
    db: Data<Db>,
    redis_conn: Data<RedisConnection>,
    current_user: ReqData<User>,
    room_id: Path<RoomId>,
    request: Json<PostRoomsStartRequestBody>,
) -> Result<Json<RoomsStartResponse>, ApiError> {
    let request = request.into_inner();
    let room_id = room_id.into_inner();

    let room = Room::get(&mut db.get_conn().await?, room_id).await?;

    let mut redis_conn = (**redis_conn).clone();

    // check if user is banned from room
    if moderation::storage::is_banned(&mut redis_conn, room.id, current_user.id)
        .await
        .map_err(Into::<ApiError>::into)?
    {
        return Err(StartRoomError::BannedFromRoom.into());
    }

    if let Some(breakout_room) = request.breakout_room {
        let config = breakout::storage::get_config(&mut redis_conn, room.id)
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
        &mut redis_conn,
        current_user.id.into(),
        room_id,
        request.breakout_room,
        request.resumption,
    )
    .await?;

    Ok(Json(RoomsStartResponse { ticket, resumption }))
}

/// API Endpoint *POST /rooms/{room_id}/start_invited*
///
/// See [`start`]
#[post("/rooms/{room_id}/start_invited")]
pub async fn start_invited(
    db: Data<Db>,
    redis_ctx: Data<RedisConnection>,
    room_id: Path<RoomId>,
    request: Json<PostRoomsStartInvitedRequestBody>,
) -> Result<ApiResponse<RoomsStartResponse>, ApiError> {
    let request = request.into_inner();
    let room_id = room_id.into_inner();

    let invite_code_as_uuid = uuid::Uuid::from_str(&request.invite_code).map_err(|_| {
        ApiError::unprocessable_entities([ValidationErrorEntry::new(
            "invite_code",
            ERROR_CODE_INVALID_VALUE,
            Some("Bad invite code format"),
        )])
    })?;

    let mut conn = db.get_conn().await?;

    let invite = Invite::get(&mut conn, InviteCodeId::from(invite_code_as_uuid)).await?;

    if !invite.active {
        return Err(ApiError::not_found());
    }

    if invite.room != room_id {
        return Err(ApiError::bad_request().with_message("Room id mismatch"));
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

    let mut redis_conn = (**redis_ctx).clone();

    if let Some(breakout_room) = request.breakout_room {
        let config = breakout::storage::get_config(&mut redis_conn, room.id)
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
        &mut redis_conn,
        Participant::Guest,
        room_id,
        request.breakout_room,
        request.resumption,
    )
    .await?;

    Ok(ApiResponse::new(RoomsStartResponse { ticket, resumption }))
}

pub trait RoomsPoliciesBuilderExt {
    fn room_guest_read_access(self, room_id: RoomId) -> Self;
    fn room_read_access(self, room_id: RoomId) -> Self;
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

pub(crate) fn associated_resource_ids(room_id: RoomId) -> impl IntoIterator<Item = ResourceId> {
    [
        room_id.resource_id(),
        room_id.resource_id().with_suffix("/invites"),
        room_id.resource_id().with_suffix("/invites/*"),
        room_id.resource_id().with_suffix("/streaming_targets"),
        room_id.resource_id().with_suffix("/streaming_targets/*"),
        room_id.resource_id().with_suffix("/start"),
        room_id.resource_id().with_suffix("/tariff"),
        room_id.resource_id().with_suffix("/event"),
        room_id.resource_id().with_suffix("/assets"),
        room_id.resource_id().with_suffix("/assets/*"),
    ]
}
