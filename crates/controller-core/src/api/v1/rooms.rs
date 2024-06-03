// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Room related API structs and Endpoints
//!
//! The defined structs are exposed to the REST API and will be serialized/deserialized. Similar
//! structs are defined in the Database crate [`opentalk_db_storage`] for database operations.

use std::str::FromStr;

use actix_web::{
    delete, get, patch, post,
    web::{self, Data, Json, Path, ReqData},
};
use kustos::{
    policies_builder::{GrantingAccess, PoliciesBuilder},
    subject::IsSubject,
    AccessMethod, Authz, Resource,
};
use opentalk_controller_utils::deletion::{Deleter, RoomDeleter};
use opentalk_database::{Db, DbConnection};
use opentalk_db_storage::{
    events::Event,
    invites::Invite,
    rooms::{self as db_rooms, Room},
    sip_configs::NewSipConfig,
    streaming_targets::get_room_streaming_targets,
    tenants::Tenant,
    users::User,
    utils::build_event_info,
};
use opentalk_keycloak_admin::KeycloakAdminClient;
use opentalk_signaling_core::{ExchangeHandle, ObjectStorage, Participant, VolatileStorage};
use opentalk_types::{
    api::{
        error::{ApiError, ValidationErrorEntry, ERROR_CODE_INVALID_VALUE},
        v1::{
            pagination::PagePaginationQuery,
            rooms::{
                DeleteRoomQuery, GetRoomEventResponse, PatchRoomsRequestBody, PostRoomsRequestBody,
                PostRoomsStartInvitedRequestBody, PostRoomsStartRequestBody, RoomResource,
                RoomsStartResponse, StartRoomError,
            },
        },
    },
    common::{features, shared_folder::SharedFolder, tariff::TariffResource},
    core::{InviteCodeId, RoomId},
};
use validator::Validate;

use super::{
    events::{get_invited_mail_recipients_for_event, CancellationNotificationValues},
    response::NoContent,
};
use crate::{
    api::{
        signaling::{
            breakout::BreakoutStorageProvider as _, moderation::ModerationStorageProvider as _,
            ticket::start_or_continue_signaling_session, SignalingModules,
        },
        v1::{events::notify_invitees_about_delete, util::require_feature, ApiResponse},
    },
    services::{MailRecipient, MailService},
    settings::SharedSettingsActix,
};

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
#[allow(clippy::too_many_arguments)]
#[delete("/rooms/{room_id}")]
pub async fn delete(
    settings: SharedSettingsActix,
    db: Data<Db>,
    storage: Data<ObjectStorage>,
    exchange_handle: Data<ExchangeHandle>,
    room_id: Path<RoomId>,
    current_user: ReqData<User>,
    current_tenant: ReqData<Tenant>,
    authz: Data<Authz>,
    query: web::Query<DeleteRoomQuery>,
    mail_service: Data<MailService>,
    kc_admin_client: Data<KeycloakAdminClient>,
) -> Result<NoContent, ApiError> {
    let room_id = room_id.into_inner();
    let current_user = current_user.into_inner();
    let current_tenant = current_tenant.into_inner();
    let settings = settings.load_full();
    let mail_service = mail_service.into_inner();

    let mut conn = db.get_conn().await?;

    let notification_values = if !query.suppress_email_notification {
        gather_mail_notification_values(&mut conn, &current_user, &current_tenant, room_id).await?
    } else {
        None
    };

    let deleter = RoomDeleter::new(
        room_id,
        query.force_delete_reference_if_external_services_fail,
    );

    deleter
        .perform(
            log::logger(),
            &mut conn,
            &authz,
            Some(current_user.id),
            exchange_handle.as_ref().clone(),
            &settings,
            &storage,
        )
        .await?;

    if let Some(notification_values) = notification_values {
        notify_invitees_about_delete(
            settings,
            notification_values,
            mail_service,
            &kc_admin_client,
        )
        .await;
    }

    Ok(NoContent)
}

async fn gather_mail_notification_values(
    conn: &mut DbConnection,
    current_user: &User,
    current_tenant: &Tenant,
    room_id: RoomId,
) -> Result<Option<CancellationNotificationValues>, ApiError> {
    let linked_event_id = match Event::get_id_for_room(conn, room_id).await? {
        Some(event) => event,
        None => return Ok(None),
    };

    let (event, _invite, room, sip_config, _is_favorite, shared_folder, _tariff) =
        Event::get_with_related_items(conn, current_user.id, linked_event_id).await?;

    let streaming_targets = get_room_streaming_targets(conn, room.id).await?;

    let invitees = get_invited_mail_recipients_for_event(conn, event.id).await?;
    let created_by_mail_recipient = MailRecipient::Registered(current_user.clone().into());

    let users_to_notify = invitees
        .into_iter()
        .chain(std::iter::once(created_by_mail_recipient))
        .collect::<Vec<_>>();

    let notification_values = CancellationNotificationValues {
        tenant: current_tenant.clone(),
        created_by: current_user.clone(),
        event,
        room,
        sip_config,
        users_to_notify,
        shared_folder: shared_folder.map(SharedFolder::from),
        streaming_targets,
    };

    Ok(Some(notification_values))
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
    volatile: Data<VolatileStorage>,
    current_user: ReqData<User>,
    room_id: Path<RoomId>,
    request: Json<PostRoomsStartRequestBody>,
) -> Result<Json<RoomsStartResponse>, ApiError> {
    let request = request.into_inner();
    let room_id = room_id.into_inner();

    let room = Room::get(&mut db.get_conn().await?, room_id).await?;

    let mut volatile = (**volatile).clone();

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

    Ok(Json(RoomsStartResponse { ticket, resumption }))
}

/// API Endpoint *POST /rooms/{room_id}/start_invited*
///
/// See [`start`]
#[post("/rooms/{room_id}/start_invited")]
pub async fn start_invited(
    db: Data<Db>,
    volatile: Data<VolatileStorage>,
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

    let mut volatile = (**volatile).clone();

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
