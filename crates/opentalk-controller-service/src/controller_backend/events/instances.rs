// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Handles event instances

use chrono::{DateTime, Utc};
use kustos::policies_builder::PoliciesBuilder;
use opentalk_controller_service_facade::RequestUser;
use opentalk_controller_utils::{event::EventExt, CaptureApiError};
use opentalk_db_storage::{
    events::{Event, EventException, EventExceptionKind, NewEventException, UpdateEventException},
    invites::Invite,
    streaming_targets::get_room_streaming_targets,
    tenants::Tenant,
    users::User,
};
use opentalk_types_api_v1::{
    error::ApiError,
    events::{
        EventAndInstanceId, EventInstance, EventInstancePath, EventInstanceQuery, EventInvitee,
        EventRoomInfo, EventStatus, EventType, GetEventInstanceResponseBody,
        GetEventInstancesCursorData, GetEventInstancesQuery, GetEventInstancesResponseBody,
        InstanceId, PatchEventInstanceBody,
    },
    Cursor,
};
use opentalk_types_common::{
    events::{invites::EventInviteStatus, EventId},
    shared_folders::SharedFolder,
    time::DateTimeTz,
    training_participation_report::TrainingParticipationReportParameterSet,
};
use rrule::RRuleSet;

use crate::{
    controller_backend::{
        events::{can_edit, DateTimeTzFromDb, EventRoomInfoExt, ONE_HUNDRED_YEARS_IN_DAYS},
        RoomsPoliciesBuilderExt,
    },
    events::{
        enrich_invitees_from_keycloak, get_invited_mail_recipients_for_event,
        notifications::{notify_invitees_about_update, UpdateNotificationValues},
        shared_folder_for_user,
    },
    util::{GetUserProfilesBatched, UserProfilesBatch},
    ControllerBackend,
};

impl ControllerBackend {
    pub(crate) async fn get_event_instances(
        &self,
        current_user: &RequestUser,
        event_id: EventId,
        GetEventInstancesQuery {
            invitees_max,
            time_min,
            time_max,
            per_page,
            after,
        }: GetEventInstancesQuery,
    ) -> Result<
        (
            GetEventInstancesResponseBody,
            Option<String>,
            Option<String>,
        ),
        CaptureApiError,
    > {
        let settings = self.settings.load();

        let per_page = per_page.unwrap_or(30).clamp(1, 100);
        let page = after.map(|c| c.page).unwrap_or(1).max(1);

        let skip = per_page as usize;
        let offset = (page - 1) as usize;

        let mut conn = self.db.get_conn().await?;

        let (
            event,
            invite,
            room,
            sip_config,
            is_favorite,
            shared_folder,
            tariff,
            training_participation_report_parameter_set,
        ) = Event::get_with_related_items(&mut conn, current_user.id, event_id).await?;

        let (invitees, invitees_truncated) =
            super::get_invitees_for_event(&settings, &mut conn, event.id, invitees_max).await?;

        let invite_status = invite
            .map(|inv| inv.status)
            .unwrap_or(EventInviteStatus::Accepted);

        let Some(rruleset) = event.to_rruleset()? else {
            return Err(ApiError::not_found().into());
        };

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

        let training_participation_report = training_participation_report_parameter_set
            .map(TrainingParticipationReportParameterSet::from);

        let current_tenant = Tenant::get(&mut conn, current_user.tenant_id).await?;
        let current_user = User::get(&mut conn, current_user.id).await?;

        drop(conn);

        let room = EventRoomInfo::from_room(&settings, room, sip_config, &tariff);

        let can_edit = can_edit(&event, &current_user);

        let shared_folder =
            shared_folder_for_user(shared_folder, event.created_by, current_user.id);

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
                training_participation_report.clone(),
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
                &settings,
                &self.kc_admin_client,
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

        Ok((
            GetEventInstancesResponseBody(event_instances),
            instances_data.before,
            instances_data.after,
        ))
    }

    pub(crate) async fn get_event_instance(
        &self,
        current_user: &RequestUser,
        EventInstancePath {
            event_id,
            instance_id,
        }: EventInstancePath,
        query: EventInstanceQuery,
    ) -> Result<GetEventInstanceResponseBody, CaptureApiError> {
        let settings = self.settings.load();
        let mut conn = self.db.get_conn().await?;

        let (
            event,
            invite,
            room,
            sip_config,
            is_favorite,
            shared_folder,
            tariff,
            training_participation_report_parameter_set,
        ) = Event::get_with_related_items(&mut conn, current_user.id, event_id).await?;
        _ = verify_recurrence_date(&event, instance_id.into())?;

        let (invitees, invitees_truncated) =
            super::get_invitees_for_event(&settings, &mut conn, event_id, query.invitees_max)
                .await?;

        let exception =
            EventException::get_for_event(&mut conn, event_id, instance_id.into()).await?;

        let users = GetUserProfilesBatched::new()
            .add(&event)
            .add(&exception)
            .fetch(&settings, &mut conn)
            .await?;

        let room = EventRoomInfo::from_room(&settings, room, sip_config, &tariff);

        let current_tenant = Tenant::get(&mut conn, current_user.tenant_id).await?;
        let current_user = User::get(&mut conn, current_user.id).await?;

        let can_edit = can_edit(&event, &current_user);

        let shared_folder =
            shared_folder_for_user(shared_folder, event.created_by, current_user.id);

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
            training_participation_report_parameter_set.map(Into::into),
        )?;

        let event_instance = EventInstance {
            invitees: enrich_invitees_from_keycloak(
                &settings,
                &self.kc_admin_client,
                &current_tenant,
                event_instance.invitees,
            )
            .await,
            ..event_instance
        };

        Ok(GetEventInstanceResponseBody(event_instance))
    }

    pub(crate) async fn patch_event_instance(
        &self,
        current_user: RequestUser,
        EventInstancePath {
            event_id,
            instance_id,
        }: EventInstancePath,
        EventInstanceQuery {
            invitees_max,
            suppress_email_notification,
        }: EventInstanceQuery,
        patch: PatchEventInstanceBody,
    ) -> Result<Option<EventInstance>, CaptureApiError> {
        if patch.is_empty() {
            return Ok(None);
        }

        let settings = self.settings.load();
        let mut conn = self.db.get_conn().await?;

        let (
            event,
            invite,
            room,
            sip_config,
            is_favorite,
            shared_folder,
            tariff,
            training_participation_report_parameter_set,
        ) = Event::get_with_related_items(&mut conn, current_user.id, event_id).await?;

        if !event.is_recurring.unwrap_or_default() {
            return Err(ApiError::not_found().into());
        }

        _ = verify_recurrence_date(&event, instance_id.into())?;

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
            super::get_invitees_for_event(&settings, &mut conn, event_id, invitees_max).await?;

        let users = GetUserProfilesBatched::new()
            .add(&event)
            .add(&exception)
            .fetch(&settings, &mut conn)
            .await?;

        let event_room_info =
            EventRoomInfo::from_room(&settings, room.clone(), sip_config.clone(), &tariff);

        let current_tenant = Tenant::get(&mut conn, current_user.tenant_id).await?;
        let current_user = User::get(&mut conn, current_user.id).await?;

        let can_edit = can_edit(&event, &current_user);

        let shared_folder =
            shared_folder_for_user(shared_folder, event.created_by, current_user.id);

        let streaming_targets = get_room_streaming_targets(&mut conn, room.id).await?;

        if !suppress_email_notification {
            let invited_users = get_invited_mail_recipients_for_event(&mut conn, event_id).await?;
            let invite_for_room =
                Invite::get_first_or_create_for_room(&mut conn, room.id, current_user.id).await?;

            let created_by = if event.created_by == current_user.id {
                current_user
            } else {
                User::get(&mut conn, event.created_by).await?
            };

            // Add the access policy for the invite code, just in case it has been created by
            // the `Invite::get_first_for_room(…)` call above. That function is not able to
            // add the policy, because it has no access to the `RoomsPoliciesBuilderExt` trait.
            let policies = PoliciesBuilder::new()
                // Grant invitee access
                .grant_invite_access(invite_for_room.id)
                .room_guest_read_access(room.id)
                .finish();
            self.authz.add_policies(policies).await?;

            let notification_values = UpdateNotificationValues {
                tenant: current_tenant.clone(),
                created_by,
                event: event.clone(),
                event_exception: Some(exception.clone()),
                room,
                sip_config,
                users_to_notify: invited_users,
                invite_for_room,
            };

            notify_invitees_about_update(
                &settings,
                notification_values,
                &self.mail_service,
                &self.kc_admin_client,
                None,
                streaming_targets,
            )
            .await;
        }

        drop(conn);

        let event_instance = create_event_instance(
            &users,
            event,
            invite
                .map(|inv| inv.status)
                .unwrap_or(EventInviteStatus::Accepted),
            is_favorite,
            Some(exception),
            event_room_info,
            instance_id,
            invitees,
            invitees_truncated,
            can_edit,
            shared_folder,
            training_participation_report_parameter_set.map(Into::into),
        )?;

        let event_instance = EventInstance {
            invitees: enrich_invitees_from_keycloak(
                &settings,
                &self.kc_admin_client,
                &current_tenant,
                event_instance.invitees,
            )
            .await,
            ..event_instance
        };

        Ok(Some(event_instance))
    }
}

struct GetPaginatedEventInstancesData {
    instances: Vec<EventInstance>,
    before: Option<String>,
    after: Option<String>,
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
    training_participation_report: Option<TrainingParticipationReportParameterSet>,
) -> opentalk_database::Result<EventInstance> {
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
        training_participation_report,
    })
}

fn patch<T>(dst: &mut T, value: Option<T>) {
    if let Some(value) = value {
        *dst = value;
    }
}

fn verify_recurrence_date(
    event: &Event,
    requested_dt: DateTime<Utc>,
) -> Result<RRuleSet, ApiError> {
    let Some(rruleset) = event.to_rruleset()? else {
        return Err(ApiError::not_found());
    };

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
    use std::time::SystemTime;

    use chrono_tz::Tz;
    use opentalk_test_util::assert_eq_json;
    use opentalk_types_api_v1::{
        events::{EventInviteeProfile, PublicInviteUserProfile},
        users::PublicUserProfile,
    };
    use opentalk_types_common::{
        events::{invites::InviteRole, EventId},
        rooms::RoomId,
        time::{TimeZone, Timestamp},
        users::{UserId, UserInfo},
    };

    use super::*;

    #[test]
    fn event_instance_serialize() {
        let unix_epoch: Timestamp = SystemTime::UNIX_EPOCH.into();
        let instance_id = unix_epoch.into();
        let event_id = EventId::nil();
        let user_profile = PublicUserProfile {
            id: UserId::nil(),
            email: "test@example.org".into(),
            user_info: UserInfo {
                title: "".parse().expect("valid user title"),
                firstname: "Test".into(),
                lastname: "Test".into(),
                display_name: "Tester".parse().expect("valid display name"),
                avatar_url: "https://example.org/avatar".into(),
            },
        };

        let instance = EventInstance {
            id: EventAndInstanceId(event_id, instance_id),
            recurring_event_id: event_id,
            instance_id,
            created_by: user_profile.clone(),
            created_at: unix_epoch,
            updated_by: user_profile.clone(),
            updated_at: unix_epoch,
            title: "Instance title".parse().expect("valid event title"),
            description: "Instance description"
                .parse()
                .expect("valid event description"),
            room: EventRoomInfo {
                id: RoomId::nil(),
                password: None,
                waiting_room: false,
                e2e_encryption: false,
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
            training_participation_report: None,
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
                    "waiting_room": false,
                    "e2e_encryption": false
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
