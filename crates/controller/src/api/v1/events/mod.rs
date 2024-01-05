// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use super::response::error::ValidationErrorEntry;
use super::response::{ApiError, NoContent, CODE_VALUE_REQUIRED};
use super::{ApiResponse, DefaultApiResult};
use crate::api::v1::response::CODE_IGNORED_VALUE;
use crate::api::v1::rooms::RoomsPoliciesBuilderExt;
use crate::api::v1::streaming_targets::{get_room_streaming_targets, insert_room_streaming_target};
use crate::api::v1::util::GetUserProfilesBatched;
use crate::services::{
    ExternalMailRecipient, MailRecipient, MailService, UnregisteredMailRecipient,
};
use crate::settings::SharedSettingsActix;
use actix_web::web::{Data, Json, Path, Query, ReqData};
use actix_web::{delete, get, patch, post, Either};
use anyhow::Context;
use chrono::{DateTime, Datelike, NaiveTime, Utc};
use chrono_tz::Tz;
use controller_settings::{Settings, TenantAssignment};
use database::{Db, DbConnection};
use db_storage::events::shared_folders::EventSharedFolder;
use db_storage::events::{
    email_invites::EventEmailInvite, Event, EventException, EventExceptionKind, EventInvite,
    NewEvent, UpdateEvent,
};
use db_storage::invites::Invite;
use db_storage::rooms::{NewRoom, Room, UpdateRoom};
use db_storage::sip_configs::{NewSipConfig, SipConfig};
use db_storage::tariffs::Tariff;
use db_storage::tenants::Tenant;
use db_storage::users::{email_to_libravatar_url, User};
use keycloak_admin::users::TenantFilter;
use keycloak_admin::KeycloakAdminClient;
use kustos::policies_builder::{GrantingAccess, PoliciesBuilder};
use kustos::prelude::{AccessMethod, IsSubject};
use kustos::{Authz, Resource, ResourceId};
use rrule::{Frequency, RRuleSet};
use serde::Deserialize;
use types::core::Timestamp;
use types::{
    api::v1::{
        events::{
            CallInInfo, DeleteEventQuery, EmailOnlyUser, EventAndInstanceId,
            EventExceptionResource, EventInvitee, EventInviteeProfile, EventOrException,
            EventResource, EventRoomInfo, EventStatus, EventType, GetEventQuery,
            GetEventsCursorData, GetEventsQuery, PatchEventBody, PatchEventQuery, PostEventsBody,
            PublicInviteUserProfile,
        },
        pagination::default_pagination_per_page,
        users::{PublicUserProfile, UnregisteredUser},
        utils::validate_recurrence_pattern,
        Cursor,
    },
    common::{
        features,
        shared_folder::{SharedFolder, SharedFolderAccess},
        streaming::{RoomStreamingTarget, StreamingTarget},
    },
    core::{DateTimeTz, EventId, EventInviteStatus, TimeZone, UserId},
};
use validator::Validate;

pub mod favorites;
pub mod instances;
pub mod invites;
pub mod shared_folder;

const LOCAL_DT_FORMAT: &str = "%Y%m%dT%H%M%S";
const ONE_HUNDRED_YEARS_IN_DAYS: usize = 36525;

pub(crate) trait DateTimeTzFromDb: Sized {
    fn maybe_from_db(utc_dt: Option<DateTime<Utc>>, tz: Option<TimeZone>) -> Option<Self>;
    fn starts_at_of(event: &Event) -> Option<Self>;
    fn ends_at_of(event: &Event) -> Option<Self>;
    fn to_datetime_tz(self) -> DateTime<Tz>;
}

impl DateTimeTzFromDb for DateTimeTz {
    /// Create a [`DateTimeTz`] from the database results
    ///
    /// Returns None if any of them are none.
    ///
    /// Only used to exceptions. To get the correct starts_at/ends_at [`DateTimeTz`] values
    /// [`DateTimeTz::starts_at_of`] and [`DateTimeTz::ends_at_of`] is used
    fn maybe_from_db(utc_dt: Option<DateTime<Utc>>, tz: Option<TimeZone>) -> Option<Self> {
        if let (Some(utc_dt), Some(tz)) = (utc_dt, tz) {
            Some(Self {
                datetime: utc_dt,
                timezone: tz,
            })
        } else {
            None
        }
    }

    /// Creates the `starts_at` DateTimeTz from an event
    fn starts_at_of(event: &Event) -> Option<Self> {
        if let (Some(dt), Some(tz)) = (event.starts_at, event.starts_at_tz) {
            Some(Self {
                datetime: dt,
                timezone: tz,
            })
        } else {
            None
        }
    }

    /// Creates the `ends_at` DateTimeTz from an event
    fn ends_at_of(event: &Event) -> Option<Self> {
        event.ends_at_of_first_occurrence().map(|(dt, tz)| Self {
            datetime: dt,
            timezone: tz,
        })
    }

    /// Combine the inner UTC time with the inner timezone
    fn to_datetime_tz(self) -> DateTime<Tz> {
        self.datetime.with_timezone(self.timezone.as_ref())
    }
}

trait EventResourceExt {
    fn from_db(exception: EventException, created_by: PublicUserProfile, can_edit: bool) -> Self;
}

impl EventResourceExt for EventExceptionResource {
    fn from_db(exception: EventException, created_by: PublicUserProfile, can_edit: bool) -> Self {
        Self {
            id: EventAndInstanceId(exception.event_id, exception.exception_date.into()),
            recurring_event_id: exception.event_id,
            instance_id: exception.exception_date.into(),
            created_by: created_by.clone(),
            created_at: exception.created_at.into(),
            updated_by: created_by,
            updated_at: exception.created_at.into(),
            title: exception.title,
            description: exception.description,
            is_all_day: exception.is_all_day,
            starts_at: DateTimeTz::maybe_from_db(exception.starts_at, exception.starts_at_tz),
            ends_at: DateTimeTz::maybe_from_db(exception.ends_at, exception.ends_at_tz),
            original_starts_at: DateTimeTz {
                datetime: exception.exception_date,
                timezone: exception.exception_date_tz,
            },
            type_: EventType::Exception,
            status: match exception.kind {
                EventExceptionKind::Modified => EventStatus::Ok,
                EventExceptionKind::Cancelled => EventStatus::Cancelled,
            },
            can_edit,
        }
    }
}

trait EventInviteeExt {
    fn from_invite_with_user(invite: EventInvite, user: User, settings: &Settings) -> Self;
    fn from_email_invite(invite: EventEmailInvite, settings: &Settings) -> Self;
}

impl EventInviteeExt for EventInvitee {
    fn from_invite_with_user(invite: EventInvite, user: User, settings: &Settings) -> EventInvitee {
        EventInvitee {
            profile: EventInviteeProfile::Registered(PublicInviteUserProfile {
                user_profile: user.to_public_user_profile(settings),
                role: invite.role,
            }),
            status: invite.status,
        }
    }

    fn from_email_invite(invite: EventEmailInvite, settings: &Settings) -> EventInvitee {
        let avatar_url = email_to_libravatar_url(&settings.avatar.libravatar_url, &invite.email);
        EventInvitee {
            profile: EventInviteeProfile::Email(EmailOnlyUser {
                email: invite.email,
                avatar_url,
            }),
            status: EventInviteStatus::Pending,
        }
    }
}

trait EventRoomInfoExt {
    fn from_room(
        settings: &Settings,
        room: Room,
        sip_config: Option<SipConfig>,
        tariff: &Tariff,
    ) -> Self;
}

impl EventRoomInfoExt for EventRoomInfo {
    /// Create a new [`EventRoomInfo`]
    ///
    /// The [`EventRoomInfo`] also contains a [`CallInInfo`] if the following conditions are true:
    /// - a telephone number is configured in the call-in settings
    /// - a [`SipConfig`] is provided
    /// - the `CallIn` feature is not disabled in the settings
    fn from_room(
        settings: &Settings,
        room: Room,
        sip_config: Option<SipConfig>,
        tariff: &Tariff,
    ) -> Self {
        let call_in_feature_is_enabled = !settings
            .defaults
            .disabled_features()
            .contains(features::CALL_IN)
            && !tariff.is_feature_disabled(features::CALL_IN);

        let mut call_in = None;

        if call_in_feature_is_enabled {
            if let (Some(call_in_config), Some(sip_config)) = (&settings.call_in, sip_config) {
                call_in = Some(CallInInfo {
                    tel: call_in_config.tel.clone(),
                    uri: None,
                    id: sip_config.sip_id.to_string(),
                    password: sip_config.password.to_string(),
                });
            }
        }

        Self {
            id: room.id,
            password: room.password,
            waiting_room: room.waiting_room,
            call_in,
        }
    }
}

/// API Endpoint `POST /events`
#[post("/events")]
pub async fn new_event(
    settings: SharedSettingsActix,
    db: Data<Db>,
    authz: Data<Authz>,
    current_user: ReqData<User>,
    new_event: Json<PostEventsBody>,
    query: Query<DeleteEventQuery>,
    mail_service: Data<MailService>,
) -> DefaultApiResult<EventResource> {
    let settings = settings.load_full();
    let current_user = current_user.into_inner();
    let new_event = new_event.into_inner();

    new_event.validate()?;

    let mut conn = db.get_conn().await?;

    // simplify logic by splitting the event creation
    // into two paths: time independent and time dependent
    let event_resource = match new_event {
        PostEventsBody {
            title,
            description,
            password,
            waiting_room,
            is_time_independent: true,
            is_all_day: None,
            starts_at: None,
            ends_at: None,
            recurrence_pattern,
            is_adhoc,
            streaming_targets,
        } if recurrence_pattern.is_empty() => {
            create_time_independent_event(
                &settings,
                &mut conn,
                current_user,
                title,
                description,
                password,
                waiting_room,
                is_adhoc,
                streaming_targets,
                query,
                mail_service,
            )
            .await?
        }
        PostEventsBody {
            title,
            description,
            password,
            waiting_room,
            is_time_independent: false,
            is_all_day: Some(is_all_day),
            starts_at: Some(starts_at),
            ends_at: Some(ends_at),
            recurrence_pattern,
            is_adhoc,
            streaming_targets,
        } => {
            create_time_dependent_event(
                &settings,
                &mut conn,
                current_user,
                title,
                description,
                password,
                waiting_room,
                is_all_day,
                starts_at,
                ends_at,
                recurrence_pattern,
                is_adhoc,
                streaming_targets,
                query,
                mail_service,
            )
            .await?
        }
        new_event => {
            let msg = if new_event.is_time_independent {
                "time independent events must not have is_all_day, starts_at, ends_at or recurrence_pattern set"
            } else {
                "time dependent events must have title, description, is_all_day, starts_at and ends_at set"
            };

            return Err(ApiError::bad_request().with_message(msg));
        }
    };

    drop(conn);

    let policies = PoliciesBuilder::new()
        .grant_user_access(event_resource.created_by.id)
        .event_read_access(event_resource.id)
        .event_write_access(event_resource.id)
        .room_read_access(event_resource.room.id)
        .room_write_access(event_resource.room.id)
        .finish();

    authz.add_policies(policies).await?;

    Ok(ApiResponse::new(event_resource))
}

async fn store_event_streaming_targets(
    conn: &mut DbConnection,
    event_id: EventId,
    streaming_targets: Option<Vec<StreamingTarget>>,
) -> Result<Option<Vec<RoomStreamingTarget>>, ApiError> {
    let room_id = Event::get(conn, event_id).await?.room;

    let streaming_targets = if let Some(streaming_targets) = streaming_targets {
        let mut room_streaming_targets: Vec<RoomStreamingTarget> = Vec::new();
        for streaming_target in streaming_targets {
            room_streaming_targets
                .push(insert_room_streaming_target(conn, room_id, streaming_target).await?);
        }
        Some(room_streaming_targets)
    } else {
        None
    };

    Ok(streaming_targets)
}

/// Part of `POST /events` endpoint
#[allow(clippy::too_many_arguments)]
async fn create_time_independent_event(
    settings: &Settings,
    conn: &mut DbConnection,
    current_user: User,
    title: String,
    description: String,
    password: Option<String>,
    waiting_room: bool,
    is_adhoc: bool,
    streaming_targets: Option<Vec<StreamingTarget>>,
    query: Query<DeleteEventQuery>,
    mail_service: Data<MailService>,
) -> Result<EventResource, ApiError> {
    let room = NewRoom {
        created_by: current_user.id,
        password,
        waiting_room,
        tenant_id: current_user.tenant_id,
    }
    .insert(conn)
    .await?;

    let sip_config = NewSipConfig::new(room.id, false).insert(conn).await?;

    let event = NewEvent {
        title,
        description,
        room: room.id,
        created_by: current_user.id,
        updated_by: current_user.id,
        is_time_independent: true,
        is_all_day: None,
        starts_at: None,
        starts_at_tz: None,
        ends_at: None,
        ends_at_tz: None,
        duration_secs: None,
        is_recurring: None,
        recurrence_pattern: None,
        is_adhoc,
        tenant_id: current_user.tenant_id,
    }
    .insert(conn)
    .await?;

    let streaming_targets =
        store_event_streaming_targets(conn, event.id, streaming_targets).await?;

    let tariff = Tariff::get_by_user_id(conn, &current_user.id).await?;

    let suppress_email_notification = is_adhoc || query.suppress_email_notification;

    if !suppress_email_notification {
        mail_service
            .send_registered_invite(
                current_user.clone(),
                event.clone(),
                room.clone(),
                Some(sip_config.clone()),
                current_user.clone(),
                None,
            )
            .await
            .context("Failed to send with MailService")?;
    }

    Ok(EventResource {
        id: event.id,
        title: event.title,
        description: event.description,
        room: EventRoomInfo::from_room(settings, room, Some(sip_config), &tariff),
        invitees_truncated: false,
        invitees: vec![],
        created_by: current_user.to_public_user_profile(settings),
        created_at: event.created_at.into(),
        updated_by: current_user.to_public_user_profile(settings),
        updated_at: event.updated_at.into(),
        is_time_independent: true,
        is_all_day: None,
        starts_at: None,
        ends_at: None,
        recurrence_pattern: vec![],
        type_: EventType::Single,
        invite_status: EventInviteStatus::Accepted,
        is_favorite: false,
        can_edit: true, // just created by the current user
        is_adhoc,
        shared_folder: None,
        streaming_targets,
    })
}

/// Part of `POST /events` endpoint
#[allow(clippy::too_many_arguments)]
async fn create_time_dependent_event(
    settings: &Settings,
    conn: &mut DbConnection,
    current_user: User,
    title: String,
    description: String,
    password: Option<String>,
    waiting_room: bool,
    is_all_day: bool,
    starts_at: DateTimeTz,
    ends_at: DateTimeTz,
    recurrence_pattern: Vec<String>,
    is_adhoc: bool,
    streaming_targets: Option<Vec<StreamingTarget>>,
    query: Query<DeleteEventQuery>,
    mail_service: Data<MailService>,
) -> Result<EventResource, ApiError> {
    let recurrence_pattern = recurrence_array_to_string(recurrence_pattern);

    let (duration_secs, ends_at_dt, ends_at_tz) =
        parse_event_dt_params(is_all_day, starts_at, ends_at, &recurrence_pattern)?;

    let room = NewRoom {
        created_by: current_user.id,
        password,
        waiting_room,
        tenant_id: current_user.tenant_id,
    }
    .insert(conn)
    .await?;

    let sip_config = NewSipConfig::new(room.id, false).insert(conn).await?;

    let event = NewEvent {
        title,
        description,
        room: room.id,
        created_by: current_user.id,
        updated_by: current_user.id,
        is_time_independent: false,
        is_all_day: Some(is_all_day),
        starts_at: Some(starts_at.to_datetime_tz()),
        starts_at_tz: Some(starts_at.timezone),
        ends_at: Some(ends_at_dt),
        ends_at_tz: Some(ends_at_tz),
        duration_secs,
        is_recurring: Some(recurrence_pattern.is_some()),
        recurrence_pattern,
        is_adhoc,
        tenant_id: current_user.tenant_id,
    }
    .insert(conn)
    .await?;

    let streaming_targets =
        store_event_streaming_targets(conn, event.id, streaming_targets).await?;

    let tariff = Tariff::get_by_user_id(conn, &current_user.id).await?;

    let suppress_email_notification = is_adhoc || query.suppress_email_notification;

    if !suppress_email_notification {
        mail_service
            .send_registered_invite(
                current_user.clone(),
                event.clone(),
                room.clone(),
                Some(sip_config.clone()),
                current_user.clone(),
                None,
            )
            .await
            .context("Failed to send with MailService")?;
    }

    Ok(EventResource {
        id: event.id,
        title: event.title,
        description: event.description,
        room: EventRoomInfo::from_room(settings, room, Some(sip_config), &tariff),
        invitees_truncated: false,
        invitees: vec![],
        created_by: current_user.to_public_user_profile(settings),
        created_at: event.created_at.into(),
        updated_by: current_user.to_public_user_profile(settings),
        updated_at: event.updated_at.into(),
        is_time_independent: event.is_time_independent,
        is_all_day: event.is_all_day,
        starts_at: Some(starts_at),
        ends_at: Some(ends_at),
        recurrence_pattern: recurrence_string_to_array(event.recurrence_pattern),
        type_: if event.is_recurring.unwrap_or_default() {
            EventType::Recurring
        } else {
            EventType::Single
        },
        invite_status: EventInviteStatus::Accepted,
        is_favorite: false,
        can_edit: true, // just created by the current user
        is_adhoc,
        shared_folder: None,
        streaming_targets,
    })
}

struct GetPaginatedEventsData {
    event_resources: Vec<EventOrException>,
    before: Option<String>,
    after: Option<String>,
}

/// API Endpoint `GET /events`
///
/// Returns a paginated list of events and their exceptions inside the given time range
///
/// See documentation of [`GetEventsQuery`] for all query options
#[get("/events")]
pub async fn get_events(
    settings: SharedSettingsActix,
    db: Data<Db>,
    kc_admin_client: Data<KeycloakAdminClient>,
    current_tenant: ReqData<Tenant>,
    current_user: ReqData<User>,
    query: Query<GetEventsQuery>,
) -> DefaultApiResult<Vec<EventOrException>> {
    let settings = settings.load_full();
    let current_user = current_user.into_inner();
    let query = query.into_inner();

    let kc_admin_client_ref = &kc_admin_client;

    let per_page = query
        .per_page
        .unwrap_or_else(default_pagination_per_page)
        .clamp(1, 100);

    let mut users = GetUserProfilesBatched::new();

    let get_events_cursor = query
        .after
        .map(|cursor| db_storage::events::GetEventsCursor {
            from_id: cursor.event_id,
            from_created_at: cursor.event_created_at.into(),
            from_starts_at: cursor.event_starts_at.map(DateTime::from),
        });

    let mut conn = db.get_conn().await?;

    let events = Event::get_all_for_user_paginated(
        &mut conn,
        &current_user,
        query.favorites,
        query.invite_status,
        query.time_min.map(DateTime::from),
        query.time_max.map(DateTime::from),
        query.adhoc,
        query.time_independent,
        get_events_cursor,
        per_page,
    )
    .await?;

    for (event, _, _, _, exceptions, _, _, _) in &events {
        users.add(event);
        users.add(exceptions);
    }

    let users = users.fetch(&settings, &mut conn).await?;

    let event_refs: Vec<&Event> = events.iter().map(|(event, ..)| event).collect();

    // Build list of event invites with user, grouped by events
    let invites_with_users_grouped_by_event = if query.invitees_max == 0 {
        // Do not query event invites if invitees_max is zero, instead create dummy value
        (0..events.len()).map(|_| Vec::new()).collect()
    } else {
        EventInvite::get_for_events(&mut conn, &event_refs).await?
    };

    // Build list of additional email event invites, grouped by events
    let email_invites_grouped_by_event = if query.invitees_max == 0 {
        // Do not query email event invites if invitees_max is zero, instead create dummy value
        (0..events.len()).map(|_| Vec::new()).collect()
    } else {
        EventEmailInvite::get_for_events(&mut conn, &event_refs).await?
    };

    drop(conn);

    type InvitesByEvent = Vec<(Vec<(EventInvite, User)>, Vec<EventEmailInvite>)>;
    let invites_grouped_by_event: InvitesByEvent = invites_with_users_grouped_by_event
        .into_iter()
        .zip(email_invites_grouped_by_event)
        .collect();

    let mut event_resources = vec![];

    let mut ret_cursor_data = None;

    for (
        (event, invite, room, sip_config, exceptions, is_favorite, shared_folder, tariff),
        (mut invites_with_user, mut email_invites),
    ) in events.into_iter().zip(invites_grouped_by_event)
    {
        ret_cursor_data = Some(GetEventsCursorData {
            event_id: event.id,
            event_created_at: event.created_at.into(),
            event_starts_at: event.starts_at.map(Timestamp::from),
        });

        let created_by = users.get(event.created_by);
        let updated_by = users.get(event.updated_by);

        let invite_status = invite
            .map(|invite| invite.status)
            .unwrap_or(EventInviteStatus::Accepted);

        let invitees_truncated = query.invitees_max == 0
            || (invites_with_user.len() + email_invites.len()) > query.invitees_max as usize;

        invites_with_user.truncate(query.invitees_max as usize);
        let email_invitees_max = query.invitees_max - invites_with_user.len().max(0) as u32;
        email_invites.truncate(email_invitees_max as usize);

        let registered_invitees_iter = invites_with_user
            .into_iter()
            .map(|(invite, user)| EventInvitee::from_invite_with_user(invite, user, &settings));

        let unregistered_invitees_iter = email_invites
            .into_iter()
            .map(|invite| EventInvitee::from_email_invite(invite, &settings));

        let invitees = registered_invitees_iter
            .chain(unregistered_invitees_iter)
            .collect();

        let starts_at = DateTimeTz::starts_at_of(&event);
        let ends_at = DateTimeTz::ends_at_of(&event);

        let can_edit = can_edit(&event, &current_user);

        let shared_folder =
            shared_folder_for_user(shared_folder, event.created_by, current_user.id);

        event_resources.push(EventOrException::Event(EventResource {
            id: event.id,
            created_by,
            created_at: event.created_at.into(),
            updated_by,
            updated_at: event.updated_at.into(),
            title: event.title,
            description: event.description,
            room: EventRoomInfo::from_room(&settings, room, sip_config, &tariff),
            invitees_truncated,
            invitees,
            is_time_independent: event.is_time_independent,
            is_all_day: event.is_all_day,
            starts_at,
            ends_at,
            recurrence_pattern: recurrence_string_to_array(event.recurrence_pattern),
            type_: if event.is_recurring.unwrap_or_default() {
                EventType::Recurring
            } else {
                EventType::Single
            },
            invite_status,
            is_favorite,
            can_edit,
            is_adhoc: event.is_adhoc,
            shared_folder,
            streaming_targets: None,
        }));

        for exception in exceptions {
            let created_by = users.get(exception.created_by);

            event_resources.push(EventOrException::Exception(
                EventExceptionResource::from_db(exception, created_by, can_edit),
            ));
        }
    }

    let events_data = GetPaginatedEventsData {
        event_resources,
        before: None,
        after: ret_cursor_data.map(|c| Cursor(c).to_base64()),
    };

    let resource_mapping_futures = events_data
        .event_resources
        .into_iter()
        .map(|resource| async {
            match resource {
                EventOrException::Event(inner) => EventOrException::Event(EventResource {
                    invitees: enrich_invitees_from_keycloak(
                        settings.clone(),
                        kc_admin_client_ref,
                        &current_tenant,
                        inner.invitees,
                    )
                    .await,
                    ..inner
                }),
                EventOrException::Exception(inner) => EventOrException::Exception(inner),
            }
        });

    let event_resources = futures::future::join_all(resource_mapping_futures).await;

    Ok(ApiResponse::new(event_resources)
        .with_cursor_pagination(events_data.before, events_data.after))
}

/// API Endpoint `GET /events/{event_id}` endpoint
///
/// Returns the event resource for the given id
#[get("/events/{event_id}")]
pub async fn get_event(
    settings: SharedSettingsActix,
    db: Data<Db>,
    kc_admin_client: Data<KeycloakAdminClient>,
    current_tenant: ReqData<Tenant>,
    current_user: ReqData<User>,
    event_id: Path<EventId>,
    query: Query<GetEventQuery>,
) -> DefaultApiResult<EventResource> {
    let settings = settings.load_full();
    let event_id = event_id.into_inner();
    let query = query.into_inner();

    let mut conn = db.get_conn().await?;

    let (event, invite, room, sip_config, is_favorite, shared_folder, tariff) =
        Event::get_with_related_items(&mut conn, current_user.id, event_id).await?;
    let room_streaming_targets = get_room_streaming_targets(&mut conn, room.id).await?;
    let (invitees, invitees_truncated) =
        get_invitees_for_event(&settings, &mut conn, event_id, query.invitees_max).await?;

    let users = GetUserProfilesBatched::new()
        .add(&event)
        .fetch(&settings, &mut conn)
        .await?;

    drop(conn);

    let starts_at = DateTimeTz::starts_at_of(&event);
    let ends_at = DateTimeTz::ends_at_of(&event);

    let can_edit = can_edit(&event, &current_user);

    let shared_folder = shared_folder_for_user(shared_folder, event.created_by, current_user.id);

    let event_resource = EventResource {
        id: event.id,
        title: event.title,
        description: event.description,
        room: EventRoomInfo::from_room(&settings, room, sip_config, &tariff),
        invitees_truncated,
        invitees,
        created_by: users.get(event.created_by),
        created_at: event.created_at.into(),
        updated_by: users.get(event.updated_by),
        updated_at: event.updated_at.into(),
        is_time_independent: event.is_time_independent,
        is_all_day: event.is_all_day,
        starts_at,
        ends_at,
        recurrence_pattern: recurrence_string_to_array(event.recurrence_pattern),
        type_: if event.is_recurring.unwrap_or_default() {
            EventType::Recurring
        } else {
            EventType::Single
        },
        invite_status: invite
            .map(|inv| inv.status)
            .unwrap_or(EventInviteStatus::Accepted),
        is_favorite,
        can_edit,
        is_adhoc: event.is_adhoc,
        shared_folder,
        streaming_targets: Some(room_streaming_targets),
    };

    let event_resource = EventResource {
        invitees: enrich_invitees_from_keycloak(
            settings,
            &kc_admin_client,
            &current_tenant,
            event_resource.invitees,
        )
        .await,
        ..event_resource
    };

    Ok(ApiResponse::new(event_resource))
}

struct UpdateNotificationValues {
    pub tenant: Tenant,
    pub created_by: User,
    pub event: Event,
    pub room: Room,
    pub sip_config: Option<SipConfig>,
    pub users_to_notify: Vec<MailRecipient>,
    pub invite_for_room: Invite,
}

/// API Endpoint `PATCH /events/{event_id}`
///
/// See documentation of [`PatchEventBody`] for more infos
///
/// Patches which modify the event in a way that would invalidate existing
/// exceptions (e.g. by changing the recurrence rule or time dependence)
/// will have all exceptions deleted
#[allow(clippy::too_many_arguments)]
#[patch("/events/{event_id}")]
pub async fn patch_event(
    settings: SharedSettingsActix,
    db: Data<Db>,
    authz: Data<Authz>,
    kc_admin_client: Data<KeycloakAdminClient>,
    current_tenant: ReqData<Tenant>,
    current_user: ReqData<User>,
    event_id: Path<EventId>,
    query: Query<PatchEventQuery>,
    patch: Json<PatchEventBody>,
    mail_service: Data<MailService>,
) -> Result<Either<ApiResponse<EventResource>, NoContent>, ApiError> {
    let patch = patch.into_inner();

    if patch.is_empty() {
        return Ok(Either::Right(NoContent));
    }

    patch.validate()?;

    let settings = settings.load_full();
    let current_tenant = current_tenant.into_inner();
    let current_user = current_user.into_inner();
    let event_id = event_id.into_inner();
    let query = query.into_inner();
    let mail_service = mail_service.into_inner();

    let send_email_notification = !query.suppress_email_notification;

    let mut conn = db.get_conn().await?;

    let (event, invite, room, sip_config, is_favorite, shared_folder, tariff) =
        Event::get_with_related_items(&mut conn, current_user.id, event_id).await?;

    let room = if patch.password.is_some() || patch.waiting_room.is_some() {
        // Update the event's room if at least one of the fields is set
        UpdateRoom {
            password: patch.password.clone(),
            waiting_room: patch.waiting_room,
        }
        .apply(&mut conn, event.room)
        .await?
    } else {
        room
    };

    let created_by = if event.created_by == current_user.id {
        current_user.clone()
    } else {
        User::get(&mut conn, event.created_by).await?
    };

    // Special case: if the patch only modifies the password do not update the event
    let event = if patch.only_modifies_room() {
        event
    } else {
        let update_event = match (event.is_time_independent, patch.is_time_independent) {
            (true, Some(false)) => {
                // The patch changes the event from an time-independent event
                // to a time dependent event
                patch_event_change_to_time_dependent(&current_user, patch)?
            }
            (true, _) | (false, Some(true)) => {
                // The patch will modify an time-independent event or
                // change an event to a time-independent event
                patch_time_independent_event(&mut conn, &current_user, &event, patch).await?
            }
            _ => {
                // The patch modifies an time dependent event
                patch_time_dependent_event(&mut conn, &current_user, &event, patch).await?
            }
        };

        update_event.apply(&mut conn, event_id).await?
    };

    let invited_users = get_invited_mail_recipients_for_event(&mut conn, event_id).await?;
    let current_user_mail_recipient = MailRecipient::Registered(current_user.clone().into());
    let users_to_notify = invited_users
        .into_iter()
        .chain(std::iter::once(current_user_mail_recipient))
        .collect::<Vec<_>>();
    let invite_for_room = Invite::get_first_for_room(&mut conn, room.id, current_user.id).await?;

    // Add the access policy for the invite code, just in case it has been created by
    // the `Invite::get_first_for_room(…)` call above. That function is not able to
    // add the policy, because it has no access to the `RoomsPoliciesBuilderExt` trait.
    let policies = PoliciesBuilder::new()
        // Grant invitee access
        .grant_invite_access(invite_for_room.id)
        .room_guest_read_access(room.id)
        .finish();
    authz.add_policies(policies).await?;

    let notification_values = UpdateNotificationValues {
        tenant: current_tenant.clone(),
        created_by: created_by.clone(),
        event: event.clone(),
        room: room.clone(),
        sip_config: sip_config.clone(),
        users_to_notify,
        invite_for_room,
    };

    let (invitees, invitees_truncated) =
        get_invitees_for_event(&settings, &mut conn, event_id, query.invitees_max).await?;

    drop(conn);

    let starts_at = DateTimeTz::starts_at_of(&event);
    let ends_at = DateTimeTz::ends_at_of(&event);

    let can_edit = can_edit(&event, &current_user);

    let shared_folder = shared_folder_for_user(shared_folder, event.created_by, current_user.id);

    let event_resource = EventResource {
        id: event.id,
        created_by: created_by.to_public_user_profile(&settings),
        created_at: event.created_at.into(),
        updated_by: current_user.to_public_user_profile(&settings),
        updated_at: event.updated_at.into(),
        title: event.title,
        description: event.description,
        room: EventRoomInfo::from_room(&settings, room, sip_config, &tariff),
        invitees_truncated,
        invitees,
        is_time_independent: event.is_time_independent,
        is_all_day: event.is_all_day,
        starts_at,
        ends_at,
        recurrence_pattern: recurrence_string_to_array(event.recurrence_pattern),
        type_: if event.is_recurring.unwrap_or_default() {
            EventType::Recurring
        } else {
            EventType::Single
        },
        invite_status: invite
            .map(|inv| inv.status)
            .unwrap_or(EventInviteStatus::Accepted),
        is_favorite,
        can_edit,
        is_adhoc: event.is_adhoc,
        shared_folder: shared_folder.clone(),
        streaming_targets: None,
    };

    if send_email_notification {
        notify_invitees_about_update(
            settings.clone(),
            notification_values,
            mail_service,
            &kc_admin_client,
            shared_folder,
        )
        .await;
    }

    let event_resource = EventResource {
        invitees: enrich_invitees_from_keycloak(
            settings,
            &kc_admin_client,
            &current_tenant,
            event_resource.invitees,
        )
        .await,
        ..event_resource
    };

    Ok(Either::Left(ApiResponse::new(event_resource)))
}

/// Part of `PATCH /events/{event_id}` (see [`patch_event`])
///
/// Notify invited users about the event update
async fn notify_invitees_about_update(
    settings: Arc<Settings>,
    notification_values: UpdateNotificationValues,
    mail_service: Arc<MailService>,
    kc_admin_client: &Data<KeycloakAdminClient>,
    shared_folder: Option<SharedFolder>,
) {
    for user in notification_values.users_to_notify {
        let invited_user = enrich_from_keycloak(
            settings.clone(),
            user,
            &notification_values.tenant,
            kc_admin_client,
        )
        .await;

        if let Err(e) = mail_service
            .send_event_update(
                notification_values.created_by.clone(),
                notification_values.event.clone(),
                notification_values.room.clone(),
                notification_values.sip_config.clone(),
                invited_user,
                notification_values.invite_for_room.id.to_string(),
                shared_folder.clone(),
            )
            .await
        {
            log::error!("Failed to send event update with MailService, {}", e);
        }
    }
}

/// Part of `PATCH /events/{event_id}` (see [`patch_event`])
///
/// Patch event which is time independent into a time dependent event
fn patch_event_change_to_time_dependent(
    current_user: &User,
    patch: PatchEventBody,
) -> Result<UpdateEvent, ApiError> {
    if let (Some(is_all_day), Some(starts_at), Some(ends_at)) =
        (patch.is_all_day, patch.starts_at, patch.ends_at)
    {
        let recurrence_pattern = recurrence_array_to_string(patch.recurrence_pattern);

        let (duration_secs, ends_at_dt, ends_at_tz) =
            parse_event_dt_params(is_all_day, starts_at, ends_at, &recurrence_pattern)?;

        Ok(UpdateEvent {
            title: patch.title,
            description: patch.description,
            updated_by: current_user.id,
            updated_at: Utc::now(),
            is_time_independent: Some(false),
            is_all_day: Some(Some(is_all_day)),
            starts_at: Some(Some(starts_at.to_datetime_tz())),
            starts_at_tz: Some(Some(starts_at.timezone)),
            ends_at: Some(Some(ends_at_dt)),
            ends_at_tz: Some(Some(ends_at_tz)),
            duration_secs: Some(duration_secs),
            is_recurring: Some(Some(recurrence_pattern.is_some())),
            recurrence_pattern: Some(recurrence_pattern),
            is_adhoc: patch.is_adhoc,
        })
    } else {
        const MSG: Option<&str> = Some("Must be provided when changing to time dependent events");

        let mut entries = vec![];

        if patch.is_all_day.is_some() {
            entries.push(ValidationErrorEntry::new(
                "is_all_day",
                CODE_VALUE_REQUIRED,
                MSG,
            ))
        }

        if patch.starts_at.is_some() {
            entries.push(ValidationErrorEntry::new(
                "starts_at",
                CODE_VALUE_REQUIRED,
                MSG,
            ))
        }

        if patch.ends_at.is_some() {
            entries.push(ValidationErrorEntry::new(
                "ends_at",
                CODE_VALUE_REQUIRED,
                MSG,
            ))
        }

        Err(ApiError::unprocessable_entities(entries))
    }
}

/// Part of `PATCH /events/{event_id}` (see [`patch_event`])
///
/// Patch event which is time dependent into a time independent event
async fn patch_time_independent_event(
    conn: &mut DbConnection,
    current_user: &User,
    event: &Event,
    patch: PatchEventBody,
) -> Result<UpdateEvent, ApiError> {
    if patch.is_all_day.is_some() || patch.starts_at.is_some() || patch.ends_at.is_some() {
        const MSG: Option<&str> = Some("Value would be ignored in this request");

        let mut entries = vec![];

        if patch.is_all_day.is_some() {
            entries.push(ValidationErrorEntry::new(
                "is_all_day",
                CODE_IGNORED_VALUE,
                MSG,
            ))
        }

        if patch.starts_at.is_some() {
            entries.push(ValidationErrorEntry::new(
                "starts_at",
                CODE_IGNORED_VALUE,
                MSG,
            ))
        }

        if patch.ends_at.is_some() {
            entries.push(ValidationErrorEntry::new(
                "ends_at",
                CODE_IGNORED_VALUE,
                MSG,
            ))
        }

        return Err(ApiError::unprocessable_entities(entries));
    }

    if event.is_recurring.unwrap_or_default() {
        // delete all exceptions as the time dependence has been removed
        EventException::delete_all_for_event(conn, event.id).await?;
    }

    Ok(UpdateEvent {
        title: patch.title,
        description: patch.description,
        updated_by: current_user.id,
        updated_at: Utc::now(),
        is_time_independent: Some(true),
        is_all_day: Some(None),
        starts_at: Some(None),
        starts_at_tz: Some(None),
        ends_at: Some(None),
        ends_at_tz: Some(None),
        duration_secs: Some(None),
        is_recurring: Some(None),
        recurrence_pattern: Some(None),
        is_adhoc: patch.is_adhoc,
    })
}

/// Part of `PATCH /events/{event_id}` (see [`patch_event`])
///
/// Patch fields on an time dependent event (without changing the time dependence field)
async fn patch_time_dependent_event(
    conn: &mut DbConnection,
    current_user: &User,
    event: &Event,
    patch: PatchEventBody,
) -> Result<UpdateEvent, ApiError> {
    let recurrence_pattern = recurrence_array_to_string(patch.recurrence_pattern);

    let is_all_day = patch.is_all_day.or(event.is_all_day).unwrap();
    let starts_at = patch
        .starts_at
        .or_else(|| DateTimeTz::starts_at_of(event))
        .unwrap();
    let ends_at = patch
        .ends_at
        .or_else(|| DateTimeTz::ends_at_of(event))
        .unwrap();

    let (duration_secs, ends_at_dt, ends_at_tz) =
        parse_event_dt_params(is_all_day, starts_at, ends_at, &recurrence_pattern)?;

    if event.is_recurring.unwrap_or_default() {
        // Delete all exceptions for recurring events as the patch may modify fields that influence the
        // timestamps at which instances (occurrences) are generated, making it impossible to match the
        // exceptions to instances
        EventException::delete_all_for_event(conn, event.id).await?;
    }

    Ok(UpdateEvent {
        title: patch.title,
        description: patch.description,
        updated_by: current_user.id,
        updated_at: Utc::now(),
        is_time_independent: Some(false),
        is_all_day: Some(Some(is_all_day)),
        starts_at: Some(Some(starts_at.to_datetime_tz())),
        starts_at_tz: Some(Some(starts_at.timezone)),
        ends_at: Some(Some(ends_at_dt)),
        ends_at_tz: Some(Some(ends_at_tz)),
        duration_secs: Some(duration_secs),
        is_recurring: Some(Some(recurrence_pattern.is_some())),
        is_adhoc: patch.is_adhoc,
        recurrence_pattern: Some(recurrence_pattern),
    })
}

struct CancellationNotificationValues {
    pub tenant: Tenant,
    pub created_by: User,
    pub event: Event,
    pub room: Room,
    pub sip_config: Option<SipConfig>,
    pub users_to_notify: Vec<MailRecipient>,
}

/// API Endpoint `DELETE /events/{event_id}`
#[delete("/events/{event_id}")]
#[allow(clippy::too_many_arguments)]
pub async fn delete_event(
    settings: SharedSettingsActix,
    db: Data<Db>,
    kc_admin_client: Data<KeycloakAdminClient>,
    current_tenant: ReqData<Tenant>,
    current_user: ReqData<User>,
    authz: Data<Authz>,
    query: Query<DeleteEventQuery>,
    event_id: Path<EventId>,
    mail_service: Data<MailService>,
) -> Result<NoContent, ApiError> {
    let settings = settings.load_full();
    let current_user = current_user.into_inner();
    let event_id = event_id.into_inner();
    let mail_service = mail_service.into_inner();

    let send_email_notification = !query.suppress_email_notification;

    let mut conn = db.get_conn().await?;

    // TODO(w.rabl) Further DB access optimization (replacing call to get_with_invite_and_room)?
    let (event, _invite, room, sip_config, _is_favorite, shared_folder, _tariff) =
        Event::get_with_related_items(&mut conn, current_user.id, event_id).await?;

    let created_by = if event.created_by == current_user.id {
        current_user
    } else {
        User::get(&mut conn, event.created_by).await?
    };

    let invited_users = get_invited_mail_recipients_for_event(&mut conn, event_id).await?;
    let created_by_mail_recipient = MailRecipient::Registered(created_by.clone().into());
    let users_to_notify = invited_users
        .into_iter()
        .chain(std::iter::once(created_by_mail_recipient))
        .collect::<Vec<_>>();

    Event::delete_by_id(&mut conn, event_id).await?;

    drop(conn);

    if send_email_notification {
        let notification_values = CancellationNotificationValues {
            tenant: current_tenant.into_inner(),
            created_by,
            event,
            room,
            sip_config,
            users_to_notify,
        };

        notify_invitees_about_delete(
            settings,
            notification_values,
            mail_service,
            &kc_admin_client,
            shared_folder.map(SharedFolder::from),
        )
        .await;
    }

    let resources = associated_resource_ids(event_id);

    authz.remove_explicit_resources(resources).await?;

    Ok(NoContent)
}

/// Part of `DELETE /events/{event_id}` (see [`delete_event`])
///
/// Notify invited users about the event deletion
async fn notify_invitees_about_delete(
    settings: Arc<Settings>,
    notification_values: CancellationNotificationValues,
    mail_service: Arc<MailService>,
    kc_admin_client: &Data<KeycloakAdminClient>,
    shared_folder: Option<SharedFolder>,
) {
    // Don't send mails for past events
    match notification_values.event.ends_at {
        Some(ends_at) if ends_at < Utc::now() => {
            return;
        }
        _ => {}
    }
    for user in notification_values.users_to_notify {
        let invited_user = enrich_from_keycloak(
            settings.clone(),
            user,
            &notification_values.tenant,
            kc_admin_client,
        )
        .await;

        if let Err(e) = mail_service
            .send_event_cancellation(
                notification_values.created_by.clone(),
                notification_values.event.clone(),
                notification_values.room.clone(),
                notification_values.sip_config.clone(),
                invited_user,
                shared_folder.clone(),
            )
            .await
        {
            log::error!("Failed to send event cancellation with MailService, {}", e);
        }
    }
}

pub(crate) fn associated_resource_ids(event_id: EventId) -> impl IntoIterator<Item = ResourceId> {
    [
        event_id.resource_id(),
        event_id.resource_id().with_suffix("/instances"),
        event_id.resource_id().with_suffix("/instances/*"),
        event_id.resource_id().with_suffix("/invites"),
        event_id.resource_id().with_suffix("/invites/*"),
        event_id.resource_id().with_suffix("/invite"),
        event_id.resource_id().with_suffix("/reschedule"),
        event_id.resource_id().with_suffix("/shared_folder"),
        ResourceId::from(format!("/users/me/event_favorites/{event_id}")),
    ]
}

#[derive(Deserialize, Validate)]
pub struct EventRescheduleBody {
    _from: DateTime<Utc>,
    _is_all_day: Option<bool>,
    _starts_at: Option<bool>,
    _ends_at: Option<bool>,
    #[validate(custom = "validate_recurrence_pattern")]
    _recurrence_pattern: Vec<String>,
}

#[post("/events/{event_id}/reschedule")]
pub async fn event_reschedule(
    _db: Data<Db>,
    _event_id: Path<EventId>,
    _body: Json<EventRescheduleBody>,
) -> actix_web::HttpResponse {
    if let Err(_e) = _body.validate() {
        // return Err(DefaultApiError::ValidationFailed);
        return actix_web::HttpResponse::NotImplemented().finish();
    }

    actix_web::HttpResponse::NotImplemented().finish()
}

async fn get_invitees_for_event(
    settings: &Settings,
    conn: &mut DbConnection,
    event_id: EventId,
    invitees_max: i64,
) -> database::Result<(Vec<EventInvitee>, bool)> {
    if invitees_max > 0 {
        // Get regular invitees up to the maximum invitee count specified.

        let (invites_with_user, total_invites_count) =
            EventInvite::get_for_event_paginated(conn, event_id, invitees_max, 1).await?;

        let mut invitees: Vec<EventInvitee> = invites_with_user
            .into_iter()
            .map(|(invite, user)| EventInvitee::from_invite_with_user(invite, user, settings))
            .collect();

        let loaded_invites_count = invitees.len() as i64;
        let mut invitees_truncated = total_invites_count > loaded_invites_count;

        // Now add email invitees until the maximum total invitee count specified is reached.

        let invitees_max = invitees_max - loaded_invites_count;
        if invitees_max > 0 {
            let (email_invites, total_email_invites_count) =
                EventEmailInvite::get_for_event_paginated(conn, event_id, invitees_max, 1).await?;

            let email_invitees: Vec<EventInvitee> = email_invites
                .into_iter()
                .map(|invite| EventInvitee::from_email_invite(invite, settings))
                .collect();

            let loaded_email_invites_count = email_invitees.len() as i64;
            invitees_truncated =
                invitees_truncated || (total_email_invites_count > loaded_email_invites_count);

            invitees.extend(email_invitees);
        }

        Ok((invitees, invitees_truncated))
    } else {
        Ok((vec![], true))
    }
}

async fn get_invited_mail_recipients_for_event(
    conn: &mut DbConnection,
    event_id: EventId,
) -> database::Result<Vec<MailRecipient>> {
    // TODO(w.rabl) Further DB access optimization (replacing call to get_for_event_paginated)?
    let (invites_with_user, _) =
        EventInvite::get_for_event_paginated(conn, event_id, i64::MAX, 1).await?;
    let user_invitees = invites_with_user
        .into_iter()
        .map(|(_, user)| MailRecipient::Registered(user.into()));

    let (email_invites, _) =
        EventEmailInvite::get_for_event_paginated(conn, event_id, i64::MAX, 1).await?;
    let email_invitees = email_invites.into_iter().map(|invitee| {
        MailRecipient::External(ExternalMailRecipient {
            email: invitee.email,
        })
    });

    let invitees = user_invitees.chain(email_invitees).collect();

    Ok(invitees)
}

async fn enrich_invitees_from_keycloak(
    settings: Arc<Settings>,
    kc_admin_client: &Data<KeycloakAdminClient>,
    current_tenant: &Tenant,
    invitees: Vec<EventInvitee>,
) -> Vec<EventInvitee> {
    let tenant_assignment = &settings.tenants.assignment;
    let invitee_mapping_futures = invitees.into_iter().map(|invitee| async move {
        if let EventInviteeProfile::Email(profile_details) = invitee.profile {
            let tenant_filter = get_tenant_filter(current_tenant, tenant_assignment);

            let user_for_email = kc_admin_client
                .get_user_for_email(tenant_filter, profile_details.email.as_ref())
                .await
                .unwrap_or_default();

            if let Some(user) = user_for_email {
                let profile_details = UnregisteredUser {
                    email: profile_details.email,
                    firstname: user.first_name,
                    lastname: user.last_name,
                    avatar_url: profile_details.avatar_url,
                };
                EventInvitee {
                    profile: EventInviteeProfile::Unregistered(profile_details),
                    ..invitee
                }
            } else {
                EventInvitee {
                    profile: EventInviteeProfile::Email(profile_details),
                    ..invitee
                }
            }
        } else {
            invitee
        }
    });
    futures::future::join_all(invitee_mapping_futures).await
}

fn get_tenant_filter<'a>(
    current_tenant: &'a Tenant,
    tenant_assignment: &'a TenantAssignment,
) -> Option<TenantFilter<'a>> {
    match tenant_assignment {
        TenantAssignment::Static { .. } => None,
        TenantAssignment::ByExternalTenantId {
            external_tenant_id_user_attribute_name,
        } => Some(TenantFilter {
            field_name: external_tenant_id_user_attribute_name,
            id: current_tenant.oidc_tenant_id.as_ref(),
        }),
    }
}

fn recurrence_array_to_string(recurrence_pattern: Vec<String>) -> Option<String> {
    if recurrence_pattern.is_empty() {
        None
    } else {
        Some(recurrence_pattern.join("\n"))
    }
}

fn recurrence_string_to_array(recurrence_pattern: Option<String>) -> Vec<String> {
    recurrence_pattern
        .map(|s| s.split('\n').map(String::from).collect())
        .unwrap_or_default()
}

fn verify_exception_dt_params(
    is_all_day: bool,
    starts_at: DateTimeTz,
    ends_at: DateTimeTz,
) -> Result<(), ApiError> {
    parse_event_dt_params(is_all_day, starts_at, ends_at, &None).map(|_| ())
}

/// parse the given event dt params
///
/// checks that the given params are valid to be put in the database
///
/// That means that:
/// - starts_at >= ends_at
/// - if is_all_day: starts_at & ends_at have their time part at 00:00
/// - bounded recurrence_pattern yields at least one result
///
/// returns the duration of the event if its recurring
/// and the appropriate ends_at datetime and timezone
fn parse_event_dt_params(
    is_all_day: bool,
    starts_at: DateTimeTz,
    ends_at: DateTimeTz,
    recurrence_pattern: &Option<String>,
) -> Result<(Option<i32>, DateTime<Tz>, TimeZone), ApiError> {
    const CODE_INVALID_EVENT: &str = "invalid_event";

    let starts_at_dt = starts_at.to_datetime_tz();
    let ends_at_dt = ends_at.to_datetime_tz();

    let duration_secs = (ends_at_dt - starts_at_dt).num_seconds();

    if duration_secs < 0 {
        return Err(ApiError::unprocessable_entity()
            .with_code(CODE_INVALID_EVENT)
            .with_message("ends_at must not be before starts_at"));
    }

    if is_all_day {
        let zero = NaiveTime::from_hms_opt(0, 0, 0).unwrap();

        if starts_at.datetime.time() != zero || ends_at.datetime.time() != zero {
            return Err(ApiError::unprocessable_entity()
                .with_code(CODE_INVALID_EVENT)
                .with_message(
                    "is_all_day requires starts_at/ends_at to be set at the start of the day",
                ));
        }
    }

    if let Some(recurrence_pattern) = &recurrence_pattern {
        let starts_at_tz = starts_at.timezone;
        let starts_at_fmt = starts_at.datetime.format(LOCAL_DT_FORMAT);

        let rrule_set =
            format!("DTSTART;TZID={starts_at_tz}:{starts_at_fmt}\n{recurrence_pattern}");
        let rrule_set = match rrule_set.parse::<RRuleSet>() {
            Ok(rrule) => rrule,
            Err(e) => {
                log::warn!("failed to parse rrule {:?}", e);
                return Err(ApiError::unprocessable_entity()
                    .with_code(CODE_INVALID_EVENT)
                    .with_message("Invalid recurrence pattern"));
            }
        };

        if rrule_set
            .get_rrule()
            .iter()
            .any(|rrule| rrule.get_freq() > Frequency::Daily)
        {
            return Err(ApiError::unprocessable_entity()
                .with_code(CODE_INVALID_EVENT)
                .with_message("Frequencies below 'DAILY' are not supported"));
        }

        // Figure out ends_at timestamp
        // Check if all RRULEs are reasonably bounded in how far they go
        let is_bounded = rrule_set.get_rrule().iter().all(|rrule| {
            if let Some(count) = rrule.get_count() {
                if count < 1000 {
                    return true;
                }
            }

            if let Some(until) = rrule.get_until() {
                if (until.naive_utc() - starts_at.datetime.naive_utc()).num_days()
                    <= ONE_HUNDRED_YEARS_IN_DAYS as i64
                {
                    return true;
                }
            }

            false
        });

        let dt_of_last_occurrence = if is_bounded {
            // For bounded RRULEs calculate the date of the last occurrence
            // Still limiting the iterations - just in case
            rrule_set
                .into_iter()
                .take(ONE_HUNDRED_YEARS_IN_DAYS)
                .last()
                .ok_or_else(|| {
                    ApiError::unprocessable_entity()
                        .with_code(CODE_INVALID_EVENT)
                        .with_message("recurrence_pattern does not yield any dates")
                })?
                .with_timezone(ends_at.timezone.as_ref())
        } else {
            // For RRULEs for which calculating the last occurrence might take too
            // long, as they run forever or into the very far future, just take a
            // date 100 years from the start date (or if invalid fall back to the chrono MAX DATE)
            starts_at
                .datetime
                .with_year(ends_at_dt.year() + 100)
                .unwrap_or(DateTime::<Utc>::MAX_UTC)
                .with_timezone(ends_at.timezone.as_ref())
        };

        Ok((
            Some(duration_secs as i32),
            dt_of_last_occurrence,
            ends_at.timezone,
        ))
    } else {
        Ok((None, ends_at.to_datetime_tz(), ends_at.timezone))
    }
}

/// calculate if `user` can edit `event`
fn can_edit(event: &Event, user: &User) -> bool {
    // Its sufficient to check if the user created the event as here isn't currently a system which allows users to
    // grant write access to event
    event.created_by == user.id
}

fn shared_folder_for_user(
    shared_folder: Option<EventSharedFolder>,
    event_created_by: UserId,
    current_user: UserId,
) -> Option<SharedFolder> {
    shared_folder.map(|f| {
        let EventSharedFolder {
            write_password,
            write_url,
            read_password,
            read_url,
            ..
        } = f;

        let read_write = if event_created_by == current_user {
            Some(SharedFolderAccess {
                url: write_url,
                password: write_password,
            })
        } else {
            None
        };

        let read = SharedFolderAccess {
            url: read_url,
            password: read_password,
        };

        SharedFolder { read, read_write }
    })
}

/// Helper trait to to reduce boilerplate in the single route handlers
///
/// Bundles multiple resources into groups.
pub trait EventPoliciesBuilderExt {
    fn event_read_access(self, event_id: EventId) -> Self;
    fn event_write_access(self, event_id: EventId) -> Self;

    fn event_invite_invitee_access(self, event_id: EventId) -> Self;
}

impl<T> EventPoliciesBuilderExt for PoliciesBuilder<GrantingAccess<T>>
where
    T: IsSubject + Clone,
{
    /// GET access to the event and related endpoints.
    /// PUT and DELETE to the event_favorites endpoint.
    fn event_read_access(self, event_id: EventId) -> Self {
        self.add_resource(event_id.resource_id(), [AccessMethod::Get])
            .add_resource(
                event_id.resource_id().with_suffix("/instances"),
                [AccessMethod::Get],
            )
            .add_resource(
                event_id.resource_id().with_suffix("/instances/*"),
                [AccessMethod::Get],
            )
            .add_resource(
                event_id.resource_id().with_suffix("/invites"),
                [AccessMethod::Get],
            )
            .add_resource(
                event_id.resource_id().with_suffix("/shared_folder"),
                [AccessMethod::Get],
            )
            .add_resource(
                format!("/users/me/event_favorites/{event_id}"),
                [AccessMethod::Put, AccessMethod::Delete],
            )
    }

    /// PATCH and DELETE to the event
    /// POST to reschedule and invites of the event
    /// PATCH to instances
    /// DELETE to invites
    fn event_write_access(self, event_id: EventId) -> Self {
        self.add_resource(
            event_id.resource_id(),
            [AccessMethod::Patch, AccessMethod::Delete],
        )
        .add_resource(
            event_id.resource_id().with_suffix("/reschedule"),
            [AccessMethod::Post],
        )
        .add_resource(
            event_id.resource_id().with_suffix("/instances/*"),
            [AccessMethod::Patch],
        )
        .add_resource(
            event_id.resource_id().with_suffix("/invites"),
            [AccessMethod::Post],
        )
        .add_resource(
            event_id.resource_id().with_suffix("/invites/*"),
            [AccessMethod::Patch, AccessMethod::Delete],
        )
        .add_resource(
            event_id.resource_id().with_suffix("/shared_folder"),
            [AccessMethod::Put, AccessMethod::Delete],
        )
    }

    /// PATCH and DELETE to event invite
    fn event_invite_invitee_access(self, event_id: EventId) -> Self {
        self.add_resource(
            format!("/events/{event_id}/invite"),
            [AccessMethod::Patch, AccessMethod::Delete],
        )
    }
}

async fn enrich_from_keycloak(
    settings: Arc<Settings>,
    recipient: MailRecipient,
    current_tenant: &Tenant,
    kc_admin_client: &Data<KeycloakAdminClient>,
) -> MailRecipient {
    let tenant_assignment = &settings.tenants.assignment;
    if let MailRecipient::External(recipient) = recipient {
        let tenant_filter = get_tenant_filter(current_tenant, tenant_assignment);

        let keycloak_user = kc_admin_client
            .get_user_for_email(tenant_filter, recipient.email.as_ref())
            .await
            .unwrap_or_default();

        if let Some(keycloak_user) = keycloak_user {
            MailRecipient::Unregistered(UnregisteredMailRecipient {
                email: recipient.email,
                first_name: keycloak_user.first_name,
                last_name: keycloak_user.last_name,
            })
        } else {
            MailRecipient::External(recipient)
        }
    } else {
        recipient
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;
    use test_util::assert_eq_json;
    use types::core::{InviteRole, RoomId, TimeZone, UserId};

    #[test]
    fn rrulset_parse_works_as_used_in_this_crate() {
        assert!(
            "DTSTART;TZID=Europe/Vienna:20230723T080000\nRRULE:FREQ=DAILY;UNTIL=20240119T100000Z"
                .parse::<RRuleSet>()
                .is_ok()
        );
        // Note the semicolon before the `\n`
        assert!(
            "DTSTART;TZID=Europe/Vienna:20230723T080000;\nRRULE:FREQ=DAILY;UNTIL=20240119T100000Z"
                .parse::<RRuleSet>()
                .is_err()
        );
    }

    #[test]
    fn event_resource_serialize() {
        let unix_epoch: Timestamp = SystemTime::UNIX_EPOCH.into();

        let user_profile = PublicUserProfile {
            id: UserId::nil(),
            email: "test@example.org".into(),
            title: "".into(),
            firstname: "Test".into(),
            lastname: "Test".into(),
            display_name: "Tester".into(),
            avatar_url: "https://example.org/avatar".into(),
        };

        let event_resource = EventResource {
            id: EventId::nil(),
            created_by: user_profile.clone(),
            created_at: unix_epoch,
            updated_by: user_profile.clone(),
            updated_at: unix_epoch,
            title: "Event title".into(),
            description: "Event description".into(),
            room: EventRoomInfo {
                id: RoomId::nil(),
                password: None,
                waiting_room: false,
                call_in: None,
            },
            invitees_truncated: false,
            invitees: vec![EventInvitee {
                profile: EventInviteeProfile::Registered(PublicInviteUserProfile {
                    user_profile,
                    role: InviteRole::Moderator,
                }),
                status: EventInviteStatus::Accepted,
            }],
            is_time_independent: false,
            is_all_day: Some(false),
            starts_at: Some(DateTimeTz {
                datetime: *unix_epoch,
                timezone: TimeZone::from(Tz::Europe__Berlin),
            }),
            ends_at: Some(DateTimeTz {
                datetime: *unix_epoch,
                timezone: TimeZone::from(Tz::Europe__Berlin),
            }),
            recurrence_pattern: vec![],
            type_: EventType::Single,
            invite_status: EventInviteStatus::Accepted,
            is_favorite: false,
            can_edit: true,
            is_adhoc: false,
            shared_folder: None,
            streaming_targets: None,
        };

        assert_eq_json!(
            event_resource,
            {
                "id": "00000000-0000-0000-0000-000000000000",
                "created_by": {
                    "id": "00000000-0000-0000-0000-000000000000",
                    "email": "test@example.org",
                    "title": "",
                    "firstname": "Test",
                    "lastname": "Test",
                    "display_name": "Tester",
                    "avatar_url": "https://example.org/avatar"
                },
                "created_at": "1970-01-01T00:00:00Z",
                "updated_by": {
                    "id": "00000000-0000-0000-0000-000000000000",
                    "email": "test@example.org",
                    "title": "",
                    "firstname": "Test",
                    "lastname": "Test",
                    "display_name": "Tester",
                    "avatar_url": "https://example.org/avatar"
                },
                "updated_at": "1970-01-01T00:00:00Z",
                "title": "Event title",
                "description": "Event description",
                "room": {
                    "id": "00000000-0000-0000-0000-000000000000",
                    "waiting_room": false
                },
                "invitees_truncated": false,
                "invitees": [
                    {
                        "profile": {
                            "kind": "registered",
                            "id": "00000000-0000-0000-0000-000000000000",
                            "email": "test@example.org",
                            "title": "",
                            "firstname": "Test",
                            "lastname": "Test",
                            "display_name": "Tester",
                            "avatar_url": "https://example.org/avatar",
                            "role": "moderator"
                        },
                        "status": "accepted"
                    }
                ],
                "is_time_independent": false,
                "is_all_day": false,
                "starts_at": {
                    "datetime": "1970-01-01T00:00:00Z",
                    "timezone": "Europe/Berlin"
                },
                "ends_at": {
                    "datetime": "1970-01-01T00:00:00Z",
                    "timezone": "Europe/Berlin"
                },
                "type": "single",
                "invite_status": "accepted",
                "is_favorite": false,
                "can_edit": true,
                "is_adhoc": false,
            }
        );
    }

    #[test]
    fn event_resource_time_independent_serialize() {
        let unix_epoch: Timestamp = SystemTime::UNIX_EPOCH.into();

        let user_profile = PublicUserProfile {
            id: UserId::nil(),
            email: "test@example.org".into(),
            title: "".into(),
            firstname: "Test".into(),
            lastname: "Test".into(),
            display_name: "Tester".into(),
            avatar_url: "https://example.org/avatar".into(),
        };

        let event_resource = EventResource {
            id: EventId::nil(),
            created_by: user_profile.clone(),
            created_at: unix_epoch,
            updated_by: user_profile.clone(),
            updated_at: unix_epoch,
            title: "Event title".into(),
            description: "Event description".into(),
            room: EventRoomInfo {
                id: RoomId::nil(),
                password: None,
                waiting_room: false,
                call_in: Some(CallInInfo {
                    tel: "030123456".into(),
                    uri: None,
                    id: "1234567890".into(),
                    password: "1234567890".into(),
                }),
            },
            invitees_truncated: false,
            invitees: vec![EventInvitee {
                profile: EventInviteeProfile::Registered(PublicInviteUserProfile {
                    user_profile,
                    role: InviteRole::User,
                }),
                status: EventInviteStatus::Accepted,
            }],
            is_time_independent: true,
            is_all_day: None,
            starts_at: None,
            ends_at: None,
            recurrence_pattern: vec![],
            type_: EventType::Single,
            invite_status: EventInviteStatus::Accepted,
            is_favorite: true,
            can_edit: false,
            is_adhoc: false,
            shared_folder: None,
            streaming_targets: None,
        };

        assert_eq_json!(
            event_resource,
            {
                "id": "00000000-0000-0000-0000-000000000000",
                "created_by": {
                    "id": "00000000-0000-0000-0000-000000000000",
                    "email": "test@example.org",
                    "title": "",
                    "firstname": "Test",
                    "lastname": "Test",
                    "display_name": "Tester",
                    "avatar_url": "https://example.org/avatar"
                },
                "created_at": "1970-01-01T00:00:00Z",
                "updated_by": {
                    "id": "00000000-0000-0000-0000-000000000000",
                    "email": "test@example.org",
                    "title": "",
                    "firstname": "Test",
                    "lastname": "Test",
                    "display_name": "Tester",
                    "avatar_url": "https://example.org/avatar"
                },
                "updated_at": "1970-01-01T00:00:00Z",
                "title": "Event title",
                "description": "Event description",
                "room": {
                    "id": "00000000-0000-0000-0000-000000000000",
                    "waiting_room": false,
                    "call_in": {
                        "tel": "030123456",
                        "id": "1234567890",
                        "password": "1234567890",
                    }
                },
                "invitees_truncated": false,
                "invitees": [
                    {
                        "profile": {
                            "kind": "registered",
                            "id": "00000000-0000-0000-0000-000000000000",
                            "email": "test@example.org",
                            "title": "",
                            "firstname": "Test",
                            "lastname": "Test",
                            "display_name": "Tester",
                            "avatar_url": "https://example.org/avatar",
                            "role": "user"
                        },
                        "status": "accepted"
                    }
                ],
                "is_time_independent": true,
                "type": "single",
                "invite_status": "accepted",
                "is_favorite": true,
                "can_edit": false,
                "is_adhoc": false,
            }
        );
    }

    #[test]
    fn event_exception_serialize() {
        let unix_epoch: Timestamp = SystemTime::UNIX_EPOCH.into();
        let instance_id = unix_epoch.into();
        let event_id = EventId::nil();
        let user_profile = PublicUserProfile {
            id: UserId::nil(),
            email: "test@example.org".into(),
            title: "".into(),
            firstname: "Test".into(),
            lastname: "Test".into(),
            display_name: "Tester".into(),
            avatar_url: "https://example.org/avatar".into(),
        };

        let instance = EventExceptionResource {
            id: EventAndInstanceId(event_id, instance_id),
            recurring_event_id: event_id,
            instance_id,
            created_by: user_profile.clone(),
            created_at: unix_epoch,
            updated_by: user_profile,
            updated_at: unix_epoch,
            title: Some("Instance title".into()),
            description: Some("Instance description".into()),
            is_all_day: Some(false),
            starts_at: Some(DateTimeTz {
                datetime: *unix_epoch,
                timezone: TimeZone::from(Tz::Europe__Berlin),
            }),
            ends_at: Some(DateTimeTz {
                datetime: *unix_epoch,
                timezone: TimeZone::from(Tz::Europe__Berlin),
            }),
            original_starts_at: DateTimeTz {
                datetime: *unix_epoch,
                timezone: TimeZone::from(Tz::Europe__Berlin),
            },
            type_: EventType::Exception,
            status: EventStatus::Ok,
            can_edit: false,
        };

        assert_eq_json!(
            instance,
            {
                "id": "00000000-0000-0000-0000-000000000000_19700101T000000Z",
                "recurring_event_id": "00000000-0000-0000-0000-000000000000",
                "instance_id": "19700101T000000Z",
                "created_by": {
                    "id": "00000000-0000-0000-0000-000000000000",
                    "email": "test@example.org",
                    "title": "",
                    "firstname": "Test",
                    "lastname": "Test",
                    "display_name": "Tester",
                    "avatar_url": "https://example.org/avatar"
                },
                "created_at": "1970-01-01T00:00:00Z",
                "updated_by": {
                    "id": "00000000-0000-0000-0000-000000000000",
                    "email": "test@example.org",
                    "title": "",
                    "firstname": "Test",
                    "lastname": "Test",
                    "display_name": "Tester",
                    "avatar_url": "https://example.org/avatar"
                },
                "updated_at": "1970-01-01T00:00:00Z",
                "title": "Instance title",
                "description": "Instance description",
                "is_all_day": false,
                "starts_at": {
                    "datetime": "1970-01-01T00:00:00Z",
                    "timezone": "Europe/Berlin"
                },
                "ends_at": {
                    "datetime": "1970-01-01T00:00:00Z",
                    "timezone": "Europe/Berlin"
                },
                "original_starts_at": {
                    "datetime": "1970-01-01T00:00:00Z",
                    "timezone": "Europe/Berlin"
                },
                "type": "exception",
                "status": "ok",
                "can_edit": false,
            }
        );
    }
}
