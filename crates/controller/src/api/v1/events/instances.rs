// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::{
    can_edit, ApiResponse, DateTimeTz, DefaultApiResult, EventInvitee, LOCAL_DT_FORMAT,
    ONE_HUNDRED_YEARS_IN_DAYS,
};
use crate::api::v1::events::{
    enrich_invitees_from_keycloak, shared_folder_for_user, DateTimeTzFromDb, EventRoomInfoExt,
};
use crate::api::v1::response::{ApiError, NoContent};
use crate::api::v1::util::{GetUserProfilesBatched, UserProfilesBatch};
use crate::settings::SharedSettingsActix;
use actix_web::web::{Data, Json, Path, Query, ReqData};
use actix_web::{get, patch, Either};
use chrono::{DateTime, Utc};
use database::Db;
use db_storage::events::{
    Event, EventException, EventExceptionKind, NewEventException, UpdateEventException,
};
use db_storage::tenants::Tenant;
use db_storage::users::User;
use keycloak_admin::KeycloakAdminClient;
use rrule::RRuleSet;
use types::api::v1::events::{
    EventAndInstanceId, EventInstance, EventInstancePath, EventInstanceQuery, EventRoomInfo,
    EventStatus, EventType, GetEventInstancesCursorData, GetEventInstancesQuery, InstanceId,
    PatchEventInstanceBody,
};
use types::api::v1::Cursor;
use types::common::shared_folder::SharedFolder;
use types::core::{EventId, EventInviteStatus};
use validator::Validate;

struct GetPaginatedEventInstancesData {
    instances: Vec<EventInstance>,
    before: Option<String>,
    after: Option<String>,
}

#[get("/events/{event_id}/instances")]
pub async fn get_event_instances(
    settings: SharedSettingsActix,
    db: Data<Db>,
    kc_admin_client: Data<KeycloakAdminClient>,
    current_tenant: ReqData<Tenant>,
    current_user: ReqData<User>,
    event_id: Path<EventId>,
    query: Query<GetEventInstancesQuery>,
) -> DefaultApiResult<Vec<EventInstance>> {
    let settings = settings.load_full();
    let event_id = event_id.into_inner();
    let GetEventInstancesQuery {
        invitees_max,
        time_min,
        time_max,
        per_page,
        after,
    } = query.into_inner();

    let per_page = per_page.unwrap_or(30).clamp(1, 100);
    let page = after.map(|c| c.page).unwrap_or(1).max(1);

    let skip = per_page as usize;
    let offset = (page - 1) as usize;

    let kc_admin_client_ref = &kc_admin_client;

    let mut conn = db.get_conn().await?;

    let (event, invite, room, sip_config, is_favorite, shared_folder, tariff) =
        Event::get_with_related_items(&mut conn, current_user.id, event_id).await?;

    let (invitees, invitees_truncated) =
        super::get_invitees_for_event(&settings, &mut conn, event.id, invitees_max).await?;

    let invite_status = invite
        .map(|inv| inv.status)
        .unwrap_or(EventInviteStatus::Accepted);

    let rruleset = build_rruleset(&event)?;

    const MONTHS_PER_YEAR: u32 = 12;

    // limit of how far into the future we calculate instances
    let max_dt = Utc::now()
        .with_timezone(&rruleset.get_dt_start().timezone())
        .checked_add_months(chrono::Months::new(40 * MONTHS_PER_YEAR))
        .expect("Could not add required duration");

    let mut iter: Box<dyn Iterator<Item = DateTime<rrule::Tz>>> =
        Box::new(rruleset.into_iter().skip_while(move |&dt| dt > max_dt));

    if let Some(time_min) = time_min {
        iter = Box::new(iter.skip_while(move |&dt| dt <= *time_min));
    }

    if let Some(time_max) = time_max {
        iter = Box::new(iter.skip_while(move |&dt| dt >= *time_max));
    }

    let datetimes: Vec<DateTime<Utc>> = iter
        .skip(skip * offset)
        .take(skip)
        .map(|dt| dt.with_timezone(&Utc))
        .collect();

    let exceptions = EventException::get_all_for_event(&mut conn, event_id, &datetimes).await?;

    let users = GetUserProfilesBatched::new()
        .add(&event)
        .add(&exceptions)
        .fetch(&settings, &mut conn)
        .await?;

    drop(conn);

    let room = EventRoomInfo::from_room(&settings, room, sip_config, &tariff);

    let can_edit = can_edit(&event, &current_user);

    let shared_folder = shared_folder_for_user(shared_folder, event.created_by, current_user.id);

    let mut exceptions = exceptions.into_iter().peekable();

    let mut instances = vec![];

    for datetime in datetimes {
        let exception = exceptions.next_if(|exception| exception.exception_date == datetime);

        let instance = create_event_instance(
            &users,
            event.clone(),
            invite_status,
            is_favorite,
            exception,
            room.clone(),
            datetime.into(),
            invitees.clone(),
            invitees_truncated,
            can_edit,
            shared_folder.clone(),
        )?;

        instances.push(instance);
    }

    let next_cursor = if !instances.is_empty() {
        Some(Cursor(GetEventInstancesCursorData { page: page + 1 }).to_base64())
    } else {
        None
    };

    let instances_data = GetPaginatedEventInstancesData {
        instances,
        before: None,
        after: next_cursor,
    };

    // Enrich the invitees for the first instance only and reuse them as all instances have the same invitees.
    let event_instances = if let Some(instance) = instances_data.instances.first() {
        let enriched_invitees = enrich_invitees_from_keycloak(
            settings,
            kc_admin_client_ref,
            &current_tenant,
            instance.invitees.clone(),
        )
        .await;

        instances_data
            .instances
            .into_iter()
            .map(|instance| EventInstance {
                invitees: enriched_invitees.clone(),
                ..instance
            })
            .collect()
    } else {
        instances_data.instances
    };

    Ok(ApiResponse::new(event_instances)
        .with_cursor_pagination(instances_data.before, instances_data.after))
}

/// API Endpoint *GET /events/{id}*
///
/// Returns the event resource for the given id
#[get("/events/{event_id}/instances/{instance_id}")]
pub async fn get_event_instance(
    settings: SharedSettingsActix,
    db: Data<Db>,
    kc_admin_client: Data<KeycloakAdminClient>,
    current_tenant: ReqData<Tenant>,
    current_user: ReqData<User>,
    path: Path<EventInstancePath>,
    query: Query<EventInstanceQuery>,
) -> DefaultApiResult<EventInstance> {
    let settings = settings.load_full();
    let EventInstancePath {
        event_id,
        instance_id,
    } = path.into_inner();
    let query = query.into_inner();

    let mut conn = db.get_conn().await?;

    let (event, invite, room, sip_config, is_favorite, shared_folder, tariff) =
        Event::get_with_related_items(&mut conn, current_user.id, event_id).await?;
    verify_recurrence_date(&event, instance_id.into())?;

    let (invitees, invitees_truncated) =
        super::get_invitees_for_event(&settings, &mut conn, event_id, query.invitees_max).await?;

    let exception = EventException::get_for_event(&mut conn, event_id, instance_id.into()).await?;

    let users = GetUserProfilesBatched::new()
        .add(&event)
        .add(&exception)
        .fetch(&settings, &mut conn)
        .await?;

    let room = EventRoomInfo::from_room(&settings, room, sip_config, &tariff);

    let can_edit = can_edit(&event, &current_user);

    let shared_folder = shared_folder_for_user(shared_folder, event.created_by, current_user.id);

    let event_instance = create_event_instance(
        &users,
        event,
        invite
            .map(|inv| inv.status)
            .unwrap_or(EventInviteStatus::Accepted),
        is_favorite,
        exception,
        room,
        instance_id,
        invitees,
        invitees_truncated,
        can_edit,
        shared_folder,
    )?;

    let event_instance = EventInstance {
        invitees: enrich_invitees_from_keycloak(
            settings,
            &kc_admin_client,
            &current_tenant,
            event_instance.invitees,
        )
        .await,
        ..event_instance
    };

    Ok(ApiResponse::new(event_instance))
}

/// API Endpoint `PATCH /events/{event_id}/{instance_id}`
///
/// Patch an instance of an recurring event. This creates oder modifies an exception for the event
/// at the point of time of the given instance_id.
///
/// Returns the patched event instance
#[patch("/events/{event_id}/instances/{instance_id}")]
#[allow(clippy::too_many_arguments)]
pub async fn patch_event_instance(
    settings: SharedSettingsActix,
    db: Data<Db>,
    kc_admin_client: Data<KeycloakAdminClient>,
    current_tenant: ReqData<Tenant>,
    current_user: ReqData<User>,
    path: Path<EventInstancePath>,
    query: Query<EventInstanceQuery>,
    patch: Json<PatchEventInstanceBody>,
) -> Result<Either<ApiResponse<EventInstance>, NoContent>, ApiError> {
    let patch = patch.into_inner();

    if patch.is_empty() {
        return Ok(Either::Right(NoContent));
    }

    patch.validate()?;

    let settings = settings.load_full();
    let EventInstancePath {
        event_id,
        instance_id,
    } = path.into_inner();

    let mut conn = db.get_conn().await?;

    let (event, invite, room, sip_config, is_favorite, shared_folder, tariff) =
        Event::get_with_related_items(&mut conn, current_user.id, event_id).await?;

    if !event.is_recurring.unwrap_or_default() {
        return Err(ApiError::not_found());
    }

    verify_recurrence_date(&event, instance_id.into())?;

    let exception = if let Some(exception) =
        EventException::get_for_event(&mut conn, event_id, instance_id.into()).await?
    {
        let is_all_day = patch
            .is_all_day
            .or(exception.is_all_day)
            .or(event.is_all_day)
            .unwrap();
        let starts_at = patch
            .starts_at
            .or_else(|| DateTimeTz::starts_at_of(&event))
            .or_else(|| DateTimeTz::maybe_from_db(exception.starts_at, exception.starts_at_tz))
            .unwrap();
        let ends_at = patch
            .ends_at
            .or_else(|| DateTimeTz::ends_at_of(&event))
            .or_else(|| DateTimeTz::maybe_from_db(exception.ends_at, exception.ends_at_tz))
            .unwrap();

        super::verify_exception_dt_params(is_all_day, starts_at, ends_at)?;

        let update_exception = UpdateEventException {
            kind: match patch.status {
                Some(EventStatus::Ok) => Some(EventExceptionKind::Modified),
                Some(EventStatus::Cancelled) => Some(EventExceptionKind::Cancelled),
                None => None,
            },
            title: patch.title.map(Some),
            description: patch.description.map(Some),
            is_all_day: patch.is_all_day.map(Some),
            starts_at: patch.starts_at.map(|dt| Some(dt.to_datetime_tz())),
            starts_at_tz: patch.starts_at.map(|dt| Some(dt.timezone)),
            ends_at: patch.ends_at.map(|dt| Some(dt.to_datetime_tz())),
            ends_at_tz: patch.ends_at.map(|dt| Some(dt.timezone)),
        };

        update_exception.apply(&mut conn, exception.id).await?
    } else {
        let is_all_day = patch.is_all_day.or(event.is_all_day).unwrap();
        let starts_at = patch
            .starts_at
            .or_else(|| DateTimeTz::starts_at_of(&event))
            .unwrap();
        let ends_at = patch
            .ends_at
            .or_else(|| DateTimeTz::ends_at_of(&event))
            .unwrap();

        super::verify_exception_dt_params(is_all_day, starts_at, ends_at)?;

        let new_exception = NewEventException {
            event_id: event.id,
            exception_date: instance_id.into(),
            exception_date_tz: event.starts_at_tz.unwrap(),
            created_by: current_user.id,
            kind: if let Some(EventStatus::Cancelled) = patch.status {
                EventExceptionKind::Cancelled
            } else {
                EventExceptionKind::Modified
            },
            title: patch.title,
            description: patch.description,
            is_all_day: patch.is_all_day,
            starts_at: patch.starts_at.map(|dt| dt.to_datetime_tz()),
            starts_at_tz: patch.starts_at.map(|dt| dt.timezone),
            ends_at: patch.ends_at.map(|dt| dt.to_datetime_tz()),
            ends_at_tz: patch.ends_at.map(|dt| dt.timezone),
        };

        new_exception.insert(&mut conn).await?
    };

    let (invitees, invitees_truncated) =
        super::get_invitees_for_event(&settings, &mut conn, event_id, query.invitees_max).await?;

    let users = GetUserProfilesBatched::new()
        .add(&event)
        .add(&exception)
        .fetch(&settings, &mut conn)
        .await?;

    let room = EventRoomInfo::from_room(&settings, room, sip_config, &tariff);

    let can_edit = can_edit(&event, &current_user);

    let shared_folder = shared_folder_for_user(shared_folder, event.created_by, current_user.id);

    drop(conn);

    let event_instance = create_event_instance(
        &users,
        event,
        invite
            .map(|inv| inv.status)
            .unwrap_or(EventInviteStatus::Accepted),
        is_favorite,
        Some(exception),
        room,
        instance_id,
        invitees,
        invitees_truncated,
        can_edit,
        shared_folder,
    )?;

    let event_instance = EventInstance {
        invitees: enrich_invitees_from_keycloak(
            settings,
            &kc_admin_client,
            &current_tenant,
            event_instance.invitees,
        )
        .await,
        ..event_instance
    };

    Ok(Either::Left(ApiResponse::new(event_instance)))
}

#[allow(clippy::too_many_arguments)]
fn create_event_instance(
    users: &UserProfilesBatch,
    mut event: Event,
    invite_status: EventInviteStatus,
    is_favorite: bool,
    exception: Option<EventException>,
    room: EventRoomInfo,
    instance_id: InstanceId,
    invitees: Vec<EventInvitee>,
    invitees_truncated: bool,
    can_edit: bool,
    shared_folder: Option<SharedFolder>,
) -> database::Result<EventInstance> {
    let mut status = EventStatus::Ok;

    let mut instance_starts_at = instance_id.into();
    let mut instance_starts_at_tz = event.starts_at_tz.unwrap();

    let mut instance_ends_at =
        instance_id + chrono::Duration::seconds(event.duration_secs.unwrap() as i64);
    let mut instance_ends_at_tz = event.ends_at_tz.unwrap();

    if let Some(exception) = exception {
        event.updated_by = exception.created_by;
        event.updated_at = exception.created_at;

        patch(&mut event.title, exception.title);
        patch(&mut event.description, exception.description);

        match exception.kind {
            EventExceptionKind::Modified => {
                // Do nothing for now
            }
            EventExceptionKind::Cancelled => status = EventStatus::Cancelled,
        }

        patch(&mut instance_starts_at, exception.starts_at);
        patch(&mut instance_starts_at_tz, exception.starts_at_tz);
        patch(
            &mut instance_ends_at,
            exception.ends_at.map(InstanceId::from),
        );
        patch(&mut instance_ends_at_tz, exception.ends_at_tz);
    }

    let created_by = users.get(event.created_by);
    let updated_by = users.get(event.updated_by);

    Ok(EventInstance {
        id: EventAndInstanceId(event.id, instance_id),
        recurring_event_id: event.id,
        instance_id,
        created_by,
        created_at: event.created_at.into(),
        updated_by,
        updated_at: event.updated_at.into(),
        title: event.title,
        description: event.description,
        room,
        invitees_truncated,
        invitees,
        is_all_day: event.is_all_day.unwrap(),
        starts_at: DateTimeTz {
            datetime: instance_starts_at,
            timezone: instance_starts_at_tz,
        },
        ends_at: DateTimeTz {
            datetime: instance_ends_at.into(),
            timezone: instance_ends_at_tz,
        },
        type_: EventType::Instance,
        status,
        invite_status,
        is_favorite,
        can_edit,
        shared_folder,
    })
}

fn patch<T>(dst: &mut T, value: Option<T>) {
    if let Some(value) = value {
        *dst = value;
    }
}

fn build_rruleset(event: &Event) -> Result<RRuleSet, ApiError> {
    // TODO add recurring check into SQL query?
    if !event.is_recurring.unwrap_or_default() {
        return Err(ApiError::not_found());
    }

    // TODO add more information to internal errors here
    let recurrence_pattern = event
        .recurrence_pattern
        .as_ref()
        .ok_or_else(ApiError::internal)?;
    let starts_at = event.starts_at.ok_or_else(ApiError::internal)?;
    let starts_at_tz = event.starts_at_tz.ok_or_else(ApiError::internal)?;

    let starts_at = starts_at
        .with_timezone(starts_at_tz.as_ref())
        .naive_local()
        .format(LOCAL_DT_FORMAT);

    let rruleset = format!("DTSTART;TZID={starts_at_tz}:{starts_at}\n{recurrence_pattern}");
    let rruleset: RRuleSet = rruleset.parse().map_err(|e| {
        log::error!("failed to parse rrule from db {}", e);
        ApiError::internal()
    })?;

    Ok(rruleset)
}

fn verify_recurrence_date(
    event: &Event,
    requested_dt: DateTime<Utc>,
) -> Result<RRuleSet, ApiError> {
    let rruleset = build_rruleset(event)?;

    let requested_dt = requested_dt.with_timezone(event.starts_at_tz.unwrap().as_ref());

    // Find date in recurrence, if it does not exist this will return a 404
    // And if it finds it it will break the loop
    let found = rruleset
        .into_iter()
        .take(ONE_HUNDRED_YEARS_IN_DAYS)
        .take_while(|x| x <= &requested_dt)
        .any(|x| x == requested_dt);

    if found {
        Ok(rruleset)
    } else {
        Err(ApiError::not_found())
    }
}

#[cfg(test)]
mod tests {
    use crate::api::v1::events::{EventInviteeProfile, PublicInviteUserProfile};

    use super::*;
    use chrono_tz::Tz;
    use std::time::SystemTime;
    use test_util::assert_eq_json;
    use types::{
        api::v1::users::PublicUserProfile,
        core::{InviteRole, RoomId, TimeZone, Timestamp, UserId},
    };

    #[test]
    fn event_instance_serialize() {
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

        let instance = EventInstance {
            id: EventAndInstanceId(event_id, instance_id),
            recurring_event_id: event_id,
            instance_id,
            created_by: user_profile.clone(),
            created_at: unix_epoch,
            updated_by: user_profile.clone(),
            updated_at: unix_epoch,
            title: "Instance title".into(),
            description: "Instance description".into(),
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
                    role: InviteRole::User,
                }),
                status: EventInviteStatus::Accepted,
            }],
            is_all_day: false,
            starts_at: DateTimeTz {
                datetime: *unix_epoch,
                timezone: TimeZone::from(Tz::Europe__Berlin),
            },
            ends_at: DateTimeTz {
                datetime: *unix_epoch,
                timezone: TimeZone::from(Tz::Europe__Berlin),
            },
            type_: EventType::Instance,
            status: EventStatus::Ok,
            invite_status: EventInviteStatus::Accepted,
            is_favorite: false,
            can_edit: false,
            shared_folder: None,
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
                            "role": "user"

                        },
                        "status": "accepted"
                    }
                ],
                "is_all_day": false,
                "starts_at": {
                    "datetime": "1970-01-01T00:00:00Z",
                    "timezone": "Europe/Berlin"
                },
                "ends_at": {
                    "datetime": "1970-01-01T00:00:00Z",
                    "timezone": "Europe/Berlin"
                },
                "type": "instance",
                "status": "ok",
                "invite_status": "accepted",
                "is_favorite": false,
                "can_edit": false,
            }
        );
    }
}
