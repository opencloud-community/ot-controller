// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use derive_more::{AsRef, Display, From, FromStr, Into};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods,
    OptionalExtension, PgSortExpressionMethods, QueryDsl, Queryable,
    expression::AsExpression,
    pg::Pg,
    prelude::*,
    sql_types::{Nullable, Record, Timestamptz, Uuid},
};
use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use opentalk_database::{DatabaseError, DbConnection, Paginate, Result};
use opentalk_diesel_newtype::DieselNewtype;
use opentalk_types_common::{
    events::{
        EventDescription, EventId, EventInfo, EventTitle,
        invites::{EventInviteStatus, InviteRole},
    },
    rooms::RoomId,
    sql_enum,
    tenants::TenantId,
    time::TimeZone,
    training_participation_report::{TimeRange, TrainingParticipationReportParameterSet},
    users::UserId,
};
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

use self::shared_folders::EventSharedFolder;
use crate::{
    rooms::Room,
    schema::{
        event_exceptions, event_favorites, event_invites, event_shared_folders,
        event_training_participation_report_parameter_sets, events, rooms, sip_configs, tariffs,
        users,
    },
    sip_configs::SipConfig,
    tariffs::Tariff,
    users::User,
    utils::HasUsers,
};

#[derive(
    AsRef,
    Display,
    From,
    FromStr,
    Into,
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    AsExpression,
    FromSqlRow,
    DieselNewtype,
)]
#[diesel(sql_type = diesel::sql_types::BigInt)]
pub struct EventSerialId(i64);

#[derive(
    AsRef,
    Display,
    From,
    FromStr,
    Into,
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    AsExpression,
    FromSqlRow,
    DieselNewtype,
)]
#[diesel(sql_type = diesel::sql_types::Uuid)]
pub struct EventExceptionId(uuid::Uuid);

#[derive(
    AsRef,
    Display,
    From,
    FromStr,
    Into,
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    AsExpression,
    FromSqlRow,
    DieselNewtype,
)]
#[diesel(sql_type = diesel::sql_types::Uuid)]
pub struct EventInviteId(uuid::Uuid);

pub mod email_invites;
pub mod shared_folders;

#[derive(
    Debug,
    Clone,
    Queryable,
    Identifiable,
    Associations,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    ToRedisArgs,
    FromRedisValue,
)]
#[diesel(table_name = events)]
#[diesel(belongs_to(User, foreign_key = created_by))]
#[diesel(belongs_to(Room, foreign_key = room))]
#[to_redis_args(serde)]
#[from_redis_value(serde)]
pub struct Event {
    pub id: EventId,
    pub id_serial: EventSerialId,
    pub title: EventTitle,
    pub description: EventDescription,
    pub room: RoomId,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_by: UserId,
    pub updated_at: DateTime<Utc>,
    pub is_time_independent: bool,
    pub is_all_day: Option<bool>,

    /// start datetime of the event
    pub starts_at: Option<DateTime<Utc>>,

    /// timezone of the start-datetime of the event
    pub starts_at_tz: Option<TimeZone>,

    /// end datetime of the event
    ///
    /// For recurring events contains the timestamp of the last occurrence
    pub ends_at: Option<DateTime<Utc>>,

    /// timezone of the ends_at datetime
    pub ends_at_tz: Option<TimeZone>,

    /// Only for recurring events, since ends_at contains the information
    /// about the last occurrence of the recurring series this duration value
    /// MUST be used to calculate the event instances length
    pub duration_secs: Option<i32>,

    pub is_recurring: Option<bool>,
    pub recurrence_pattern: Option<String>,

    pub is_adhoc: bool,

    pub tenant_id: TenantId,

    pub revision: i32,

    pub show_meeting_details: bool,
}

impl Event {
    /// Returns the ends_at value of the first occurrence of the event
    pub fn ends_at_of_first_occurrence(&self) -> Option<(DateTime<Utc>, TimeZone)> {
        if self.is_recurring.unwrap_or_default() {
            // Recurring events have the last occurrence of the recurrence saved in the ends_at fields
            // So we get the starts_at_dt and add the duration_secs field to it
            if let (Some(starts_at_dt), Some(dur), Some(tz)) =
                (self.starts_at, self.duration_secs, self.ends_at_tz)
            {
                Some((starts_at_dt + chrono::Duration::seconds(i64::from(dur)), tz))
            } else {
                None
            }
        } else if let (Some(dt), Some(tz)) = (self.ends_at, self.ends_at_tz) {
            // Non recurring events just directly use the ends_at field from the db
            Some((dt, tz))
        } else {
            None
        }
    }
}

pub struct EventAndEncryption<'a>(pub &'a Event, pub bool);

impl<'a> From<EventAndEncryption<'a>> for EventInfo {
    fn from(value: EventAndEncryption<'a>) -> Self {
        let EventAndEncryption(event, e2e_encryption) = value;
        EventInfo {
            id: event.id,
            room_id: event.room,
            title: event.title.clone(),
            is_adhoc: event.is_adhoc,
            meeting_details: None,
            e2e_encryption,
        }
    }
}

impl HasUsers for &Event {
    fn populate(self, dst: &mut Vec<UserId>) {
        dst.push(self.created_by);
        dst.push(self.updated_by);
    }
}

pub struct GetEventsCursor {
    pub from_id: EventId,
    pub from_created_at: DateTime<Utc>,
    pub from_starts_at: Option<DateTime<Utc>>,
}

impl GetEventsCursor {
    pub fn from_last_event_in_query(event: &Event) -> Self {
        Self {
            from_id: event.id,
            from_created_at: event.created_at,
            from_starts_at: event.starts_at,
        }
    }
}

impl Event {
    #[tracing::instrument(err, skip_all)]
    pub async fn get(conn: &mut DbConnection, event_id: EventId) -> Result<Event> {
        let query = events::table
            .inner_join(users::table.on(users::id.eq(events::created_by)))
            .select(events::all_columns)
            .filter(events::id.eq(event_id))
            .filter(users::disabled_since.is_null());

        let event = query.first(conn).await?;

        Ok(event)
    }

    pub async fn get_all_with_creator(conn: &mut DbConnection) -> Result<Vec<(EventId, UserId)>> {
        let events = events::table
            .inner_join(users::table.on(users::id.eq(events::created_by)))
            .select((events::id, events::created_by))
            .filter(users::disabled_since.is_null())
            .load(conn)
            .await?;

        Ok(events)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn get_all_that_ended_before_including_rooms(
        conn: &mut DbConnection,
        date: DateTime<Utc>,
    ) -> Result<Vec<(EventId, RoomId)>> {
        events::table
            .inner_join(users::table.on(users::id.eq(events::created_by)))
            .select((events::id, events::room))
            .filter(events::ends_at.le(date))
            .filter(events::is_recurring.ne(true))
            .filter(users::disabled_since.is_null())
            .load(conn)
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn get_all_adhoc_created_before_including_rooms(
        conn: &mut DbConnection,
        date: DateTime<Utc>,
    ) -> Result<Vec<(EventId, RoomId)>> {
        events::table
            .inner_join(users::table.on(users::id.eq(events::created_by)))
            .select((events::id, events::room))
            .filter(events::created_at.le(date))
            .filter(events::is_adhoc.eq(true))
            .filter(users::disabled_since.is_null())
            .load(conn)
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn get_all_for_creator_including_rooms(
        conn: &mut DbConnection,
        created_by: UserId,
    ) -> Result<Vec<(EventId, RoomId)>> {
        events::table
            .inner_join(users::table.on(users::id.eq(events::created_by)))
            .select((events::id, events::room))
            .filter(events::created_by.eq(created_by))
            .filter(users::disabled_since.is_null())
            .load(conn)
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn get_all_finite_recurring(conn: &mut DbConnection) -> Result<Vec<Event>> {
        events::table
            .inner_join(users::table.on(users::id.eq(events::created_by)))
            .select(events::all_columns)
            .filter(events::is_recurring.eq(true))
            .filter(
                events::recurrence_pattern
                    .ilike("%UNTIL%")
                    .or(events::recurrence_pattern.ilike("%COUNT%")),
            )
            .filter(users::disabled_since.is_null())
            .load(conn)
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn get_all_updated_by_user(
        conn: &mut DbConnection,
        updated_by: UserId,
    ) -> Result<Vec<Event>> {
        events::table
            .inner_join(users::table.on(users::id.eq(events::created_by)))
            .select(events::all_columns)
            .filter(events::updated_by.eq(updated_by))
            .filter(users::disabled_since.is_null())
            .load(conn)
            .await
            .map_err(Into::into)
    }

    pub async fn get_all_with_invitee(
        conn: &mut DbConnection,
    ) -> Result<Vec<(EventId, RoomId, UserId)>> {
        let events = events::table
            .inner_join(users::table.on(users::id.eq(events::created_by)))
            .inner_join(event_invites::table.on(event_invites::event_id.eq(events::id)))
            .select((events::id, events::room, event_invites::invitee))
            .filter(users::disabled_since.is_null())
            .load(conn)
            .await?;

        Ok(events)
    }

    #[tracing::instrument(err, skip_all)]
    #[allow(clippy::type_complexity)]
    pub async fn get_with_related_items(
        conn: &mut DbConnection,
        user_id: UserId,
        event_id: EventId,
    ) -> Result<(
        Event,
        Option<EventInvite>,
        Room,
        Option<SipConfig>,
        bool,
        Option<EventSharedFolder>,
        Tariff,
        Option<EventTrainingParticipationReportParameterSet>,
    )> {
        let query =
            events::table
                .left_join(
                    event_invites::table.on(event_invites::event_id
                        .eq(events::id)
                        .and(event_invites::invitee.eq(user_id))),
                )
                .left_join(
                    event_favorites::table.on(event_favorites::event_id
                        .eq(events::id)
                        .and(event_favorites::user_id.eq(user_id))),
                )
                .left_join(
                    event_shared_folders::table.on(event_shared_folders::event_id.eq(events::id)),
                )
                .inner_join(rooms::table.on(events::room.eq(rooms::id)))
                .left_join(sip_configs::table.on(rooms::id.eq(sip_configs::room)))
                .inner_join(users::table.on(users::id.eq(events::created_by)))
                .inner_join(tariffs::table.on(tariffs::id.eq(users::tariff_id)))
                .left_join(
                    event_training_participation_report_parameter_sets::table
                        .on(event_training_participation_report_parameter_sets::event_id
                            .eq(events::id)),
                )
                .select((
                    events::all_columns,
                    event_invites::all_columns.nullable(),
                    rooms::all_columns,
                    sip_configs::all_columns.nullable(),
                    event_favorites::user_id.nullable().is_not_null(),
                    event_shared_folders::all_columns.nullable(),
                    tariffs::all_columns,
                    event_training_participation_report_parameter_sets::all_columns.nullable(),
                ))
                .filter(users::disabled_since.is_null())
                .filter(events::id.eq(event_id));
        Ok(query.first(conn).await?)
    }

    #[tracing::instrument(err, skip_all)]
    #[allow(clippy::type_complexity)]
    pub async fn get_with_room(
        conn: &mut DbConnection,
        event_id: EventId,
    ) -> Result<(Event, Room, Option<SipConfig>)> {
        let query = events::table
            .inner_join(users::table.on(users::id.eq(events::created_by)))
            .inner_join(rooms::table.on(events::room.eq(rooms::id)))
            .left_join(sip_configs::table.on(rooms::id.eq(sip_configs::room)))
            .select((
                events::all_columns,
                rooms::all_columns,
                sip_configs::all_columns.nullable(),
            ))
            .filter(events::id.eq(event_id))
            .filter(users::disabled_since.is_null());

        let (event, room, sip_config) = query.first(conn).await?;

        Ok((event, room, sip_config))
    }

    #[tracing::instrument(err, skip_all)]
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub async fn get_all_for_user_paginated(
        conn: &mut DbConnection,
        user: &User,
        only_favorites: bool,
        invite_status_filter: Vec<EventInviteStatus>,
        time_min: Option<DateTime<Utc>>,
        time_max: Option<DateTime<Utc>>,
        created_before: Option<DateTime<Utc>>,
        created_after: Option<DateTime<Utc>>,
        adhoc: Option<bool>,
        time_independent: Option<bool>,
        cursor: Option<GetEventsCursor>,
        limit: i64,
    ) -> Result<
        Vec<(
            Event,
            Option<EventInvite>,
            Room,
            Option<SipConfig>,
            Vec<EventException>,
            bool,
            Option<EventSharedFolder>,
            Tariff,
            Option<TrainingParticipationReportParameterSet>,
        )>,
    > {
        // Filter applied to all events which validates that the event is either created by
        // the given user or a invite to the event exists for the user
        let event_related_to_user_id = events::created_by
            .eq(user.id)
            .or(event_invites::invitee.eq(user.id));

        // Create query which select events and joins into the room of the event
        let mut query = events::table
            .left_join(
                event_invites::table.on(event_invites::event_id
                    .eq(events::id)
                    .and(event_invites::invitee.eq(user.id))),
            )
            .left_join(
                event_favorites::table.on(event_favorites::event_id
                    .eq(events::id)
                    .and(event_favorites::user_id.eq(user.id))),
            )
            .left_join(
                event_shared_folders::table.on(event_shared_folders::event_id.eq(events::id)),
            )
            .inner_join(rooms::table)
            .left_join(sip_configs::table.on(rooms::id.eq(sip_configs::room)))
            .inner_join(users::table.on(users::id.eq(events::created_by)))
            .inner_join(tariffs::table.on(tariffs::id.eq(users::tariff_id)))
            .select((
                events::all_columns,
                event_invites::all_columns.nullable(),
                rooms::all_columns,
                sip_configs::all_columns.nullable(),
                event_favorites::user_id.nullable().is_not_null(),
                event_shared_folders::all_columns.nullable(),
                tariffs::all_columns,
            ))
            .filter(events::tenant_id.eq(user.tenant_id))
            .filter(event_related_to_user_id)
            .filter(users::disabled_since.is_null())
            .order_by(events::starts_at.nullable().asc().nulls_first())
            .then_order_by(events::created_at.asc())
            .then_order_by(events::id)
            .limit(limit)
            .into_boxed::<Pg>();

        // Tuples/Composite types are ordered by lexical ordering
        if let Some(cursor) = cursor {
            if let Some(from_starts_at) = cursor.from_starts_at {
                let expr =
                    AsExpression::<Record<(Nullable<Timestamptz>,Timestamptz, Uuid)>>::as_expression((
                        events::starts_at,
                        events::created_at,
                        events::id
                    ));

                query =
                    query.filter(expr.gt((from_starts_at, cursor.from_created_at, cursor.from_id)));
            } else {
                let expr = AsExpression::<Record<(Timestamptz, Uuid)>>::as_expression((
                    events::created_at,
                    events::id,
                ));

                query = query.filter(expr.gt((cursor.from_created_at, cursor.from_id)));
            }
        }

        // Add filters to query depending on the time_(min/max) parameters
        match (time_min, time_max) {
            (Some(time_min), Some(time_max)) => {
                // we have an overlap if any of these conditions matches:
                // - starts_at is between time_min and time_max
                // - ends_at is between time_min and time_max
                // - time_min is between starts_at and ends_at
                // - time_max is between starts_at and ends_at
                query = query.filter(
                    events::starts_at
                        .between(time_min, time_max)
                        .or(events::ends_at.between(time_min, time_max))
                        .or(time_min
                            .into_sql::<Nullable<Timestamptz>>()
                            .between(events::starts_at, events::ends_at))
                        .or(time_max
                            .into_sql::<Nullable<Timestamptz>>()
                            .between(events::starts_at, events::ends_at)),
                );
            }
            (Some(time_min), None) => {
                query = query.filter(events::ends_at.ge(time_min));
            }
            (None, Some(time_max)) => {
                query = query.filter(events::starts_at.le(time_max));
            }
            (None, None) => {
                // no filters to apply
            }
        }

        if let Some(created_before) = created_before {
            query = query.filter(events::created_at.le(created_before));
        }

        if let Some(created_after) = created_after {
            query = query.filter(events::created_at.ge(created_after));
        }

        if only_favorites {
            query = query.filter(event_favorites::user_id.is_not_null());
        }

        if let Some(is_adhoc) = adhoc {
            query = query.filter(events::is_adhoc.eq(is_adhoc));
        }

        if let Some(is_time_independent) = time_independent {
            query = query.filter(events::is_time_independent.eq(is_time_independent));
        }

        if !invite_status_filter.is_empty() {
            if invite_status_filter.contains(&EventInviteStatus::Accepted) {
                // edge case to allow event creators to filter created events by 'accepted'
                query = query.filter(
                    event_invites::status
                        .eq_any(invite_status_filter)
                        .or(event_invites::status.is_null()),
                );
            } else {
                query = query.filter(event_invites::status.eq_any(invite_status_filter));
            }
        }

        let events_with_invite_and_room: Vec<(
            Event,
            Option<EventInvite>,
            Room,
            Option<SipConfig>,
            bool,
            Option<EventSharedFolder>,
            Tariff,
        )> = query.load(conn).await?;

        let mut events_with_invite_room_and_exceptions =
            Vec::with_capacity(events_with_invite_and_room.len());

        for (event, invite, room, sip_config, is_favorite, shared_folders, tariff) in
            events_with_invite_and_room
        {
            let exceptions = if event.is_recurring.unwrap_or_default() {
                event_exceptions::table
                    .filter(event_exceptions::event_id.eq(event.id))
                    .load(conn)
                    .await?
            } else {
                vec![]
            };
            // TODO: remove this
            let training_participation_report_parameter_set = None;

            events_with_invite_room_and_exceptions.push((
                event,
                invite,
                room,
                sip_config,
                exceptions,
                is_favorite,
                shared_folders,
                tariff,
                training_participation_report_parameter_set,
            ));
        }

        Ok(events_with_invite_room_and_exceptions)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn delete_by_id(conn: &mut DbConnection, event_id: EventId) -> Result<()> {
        diesel::delete(events::table)
            .filter(events::id.eq(event_id))
            .execute(conn)
            .await?;

        Ok(())
    }

    /// Returns the [`Event`] in the given [`RoomId`].
    #[tracing::instrument(err, skip_all)]
    pub async fn get_for_room(conn: &mut DbConnection, room_id: RoomId) -> Result<Option<Event>> {
        let event = events::table
            .inner_join(users::table.on(users::id.eq(events::created_by)))
            .select(events::all_columns)
            .filter(events::room.eq(room_id))
            .filter(users::disabled_since.is_null())
            .first::<Event>(conn)
            .await
            .optional()?;
        Ok(event)
    }

    /// Returns a [`EventId`] for the given [`RoomId`].
    #[tracing::instrument(err, skip_all)]
    pub async fn get_id_for_room(
        conn: &mut DbConnection,
        room_id: RoomId,
    ) -> Result<Option<EventId>> {
        let query = events::table
            .inner_join(users::table.on(users::id.eq(events::created_by)))
            .select(events::id)
            .filter(events::room.eq(room_id))
            .filter(users::disabled_since.is_null());

        let events = query.first(conn).await.optional()?;

        Ok(events)
    }

    /// Deletes all [`Event`]s in a given [`RoomId`]
    ///
    /// Fastpath for deleting multiple events in room
    #[tracing::instrument(err, skip_all)]
    pub async fn delete_for_room(conn: &mut DbConnection, room_id: RoomId) -> Result<()> {
        diesel::delete(events::table)
            .filter(events::room.eq(room_id))
            .execute(conn)
            .await?;

        Ok(())
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = events)]
pub struct NewEvent {
    pub title: EventTitle,
    pub description: EventDescription,
    pub room: RoomId,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub is_time_independent: bool,
    pub is_all_day: Option<bool>,
    pub starts_at: Option<DateTime<Tz>>,
    pub starts_at_tz: Option<TimeZone>,
    pub ends_at: Option<DateTime<Tz>>,
    pub ends_at_tz: Option<TimeZone>,
    pub duration_secs: Option<i32>,
    pub is_recurring: Option<bool>,
    pub recurrence_pattern: Option<String>,
    pub is_adhoc: bool,
    pub tenant_id: TenantId,
    pub show_meeting_details: bool,
}

impl NewEvent {
    #[tracing::instrument(err, skip_all)]
    pub async fn insert(self, conn: &mut DbConnection) -> Result<Event> {
        let query = self.insert_into(events::table);

        let event = query.get_result(conn).await?;

        Ok(event)
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = events)]
pub struct UpdateEvent {
    pub title: Option<EventTitle>,
    pub description: Option<EventDescription>,
    pub updated_by: UserId,
    pub updated_at: DateTime<Utc>,
    pub is_time_independent: Option<bool>,
    pub is_all_day: Option<Option<bool>>,
    pub starts_at: Option<Option<DateTime<Tz>>>,
    pub starts_at_tz: Option<Option<TimeZone>>,
    pub ends_at: Option<Option<DateTime<Tz>>>,
    pub ends_at_tz: Option<Option<TimeZone>>,
    pub duration_secs: Option<Option<i32>>,
    pub is_recurring: Option<Option<bool>>,
    pub recurrence_pattern: Option<Option<String>>,
    pub is_adhoc: Option<bool>,
    pub show_meeting_details: Option<bool>,
}

impl UpdateEvent {
    #[tracing::instrument(err, skip_all)]
    pub async fn apply(self, conn: &mut DbConnection, event_id: EventId) -> Result<Event> {
        let query = diesel::update(events::table)
            .filter(events::id.eq(event_id))
            .set((self, events::revision.eq(events::revision + 1)))
            .returning(events::all_columns);

        let event = query.get_result(conn).await?;

        Ok(event)
    }
}

sql_enum!(
    EventExceptionKind,
    "event_exception_kind",
    EventExceptionKindType,
    {
        Modified = b"modified",
        Cancelled = b"cancelled",
    }
);

#[derive(Debug, Clone, Queryable, Identifiable, Associations)]
#[diesel(table_name = event_exceptions)]
#[diesel(belongs_to(Event, foreign_key = event_id))]
#[diesel(belongs_to(User, foreign_key = created_by))]
pub struct EventException {
    pub id: EventExceptionId,
    pub event_id: EventId,
    pub exception_date: DateTime<Utc>,
    pub exception_date_tz: TimeZone,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub kind: EventExceptionKind,
    pub title: Option<EventTitle>,
    pub description: Option<EventDescription>,
    pub is_all_day: Option<bool>,
    pub starts_at: Option<DateTime<Utc>>,
    pub starts_at_tz: Option<TimeZone>,
    pub ends_at: Option<DateTime<Utc>>,
    pub ends_at_tz: Option<TimeZone>,
}

impl HasUsers for &EventException {
    fn populate(self, dst: &mut Vec<UserId>) {
        dst.push(self.created_by);
    }
}

impl EventException {
    #[tracing::instrument(err, skip_all)]
    pub async fn get_for_event(
        conn: &mut DbConnection,
        event_id: EventId,
        datetime: DateTime<Utc>,
    ) -> Result<Option<EventException>> {
        let query = event_exceptions::table.filter(
            event_exceptions::event_id
                .eq(event_id)
                .and(event_exceptions::exception_date.eq(datetime)),
        );

        let exceptions = query.first(conn).await.optional()?;

        Ok(exceptions)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn get_all_for_event(
        conn: &mut DbConnection,
        event_id: EventId,
        datetimes: &[DateTime<Utc>],
    ) -> Result<Vec<EventException>> {
        let query = event_exceptions::table.filter(
            event_exceptions::event_id
                .eq(event_id)
                .and(event_exceptions::exception_date.eq_any(datetimes)),
        );

        let exceptions = query.load(conn).await.optional()?.unwrap_or_default();

        Ok(exceptions)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn delete_all_for_event(conn: &mut DbConnection, event_id: EventId) -> Result<()> {
        let query =
            diesel::delete(event_exceptions::table).filter(event_exceptions::event_id.eq(event_id));

        query.execute(conn).await?;

        Ok(())
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = event_exceptions)]
pub struct NewEventException {
    pub event_id: EventId,
    pub exception_date: DateTime<Utc>,
    pub exception_date_tz: TimeZone,
    pub created_by: UserId,
    pub kind: EventExceptionKind,
    pub title: Option<EventTitle>,
    pub description: Option<EventDescription>,
    pub is_all_day: Option<bool>,
    pub starts_at: Option<DateTime<Tz>>,
    pub starts_at_tz: Option<TimeZone>,
    pub ends_at: Option<DateTime<Tz>>,
    pub ends_at_tz: Option<TimeZone>,
}

impl NewEventException {
    #[tracing::instrument(err, skip_all)]
    pub async fn insert(self, conn: &mut DbConnection) -> Result<EventException> {
        let query = self.insert_into(event_exceptions::table);

        let event_exception = query.get_result(conn).await?;

        Ok(event_exception)
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = event_exceptions)]
pub struct UpdateEventException {
    pub kind: Option<EventExceptionKind>,
    pub title: Option<Option<EventTitle>>,
    pub description: Option<Option<EventDescription>>,
    pub is_all_day: Option<Option<bool>>,
    pub starts_at: Option<Option<DateTime<Tz>>>,
    pub starts_at_tz: Option<Option<TimeZone>>,
    pub ends_at: Option<Option<DateTime<Tz>>>,
    pub ends_at_tz: Option<Option<TimeZone>>,
}

impl UpdateEventException {
    #[tracing::instrument(err, skip_all)]
    pub async fn apply(
        self,
        conn: &mut DbConnection,
        event_exception_id: EventExceptionId,
    ) -> Result<EventException> {
        let query = diesel::update(event_exceptions::table)
            .filter(event_exceptions::id.eq(event_exception_id))
            .set(self)
            .returning(event_exceptions::all_columns);

        let exception = query.get_result(conn).await?;

        Ok(exception)
    }
}

#[derive(Debug, Queryable, Identifiable, Associations)]
#[diesel(table_name = event_invites)]
#[diesel(belongs_to(Event, foreign_key = event_id))]
#[diesel(belongs_to(User, foreign_key = invitee))]
pub struct EventInvite {
    pub id: EventInviteId,
    pub event_id: EventId,
    pub invitee: UserId,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub status: EventInviteStatus,
    pub role: InviteRole,
}

impl EventInvite {
    #[tracing::instrument(err, skip_all)]
    pub async fn get_for_events(
        conn: &mut DbConnection,
        events: &[&Event],
    ) -> Result<Vec<Vec<(EventInvite, User)>>> {
        conn.transaction(|conn| {
            async move {
                let invites: Vec<EventInvite> =
                    EventInvite::belonging_to(events).load(conn).await?;
                let mut user_ids: Vec<UserId> = invites.iter().map(|x| x.invitee).collect();
                // Small optimization to filter out duplicates
                user_ids.sort_unstable();
                user_ids.dedup();

                let users = User::get_all_by_ids(conn, &user_ids).await?;

                let invites_by_event: Vec<Vec<EventInvite>> = invites.grouped_by(events);
                let mut invites_with_users_by_event = Vec::with_capacity(events.len());

                for invites in invites_by_event {
                    let mut invites_with_users = Vec::with_capacity(invites.len());

                    for invite in invites {
                        let user = users
                            .iter()
                            .find(|user| user.id == invite.invitee)
                            .ok_or_else(|| DatabaseError::Custom {
                                message: "bug: user invite invitee missing".to_owned(),
                            })?;

                        invites_with_users.push((invite, user.clone()))
                    }

                    invites_with_users_by_event.push(invites_with_users);
                }

                Ok(invites_with_users_by_event)
            }
            .scope_boxed()
        })
        .await
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn get_for_event_paginated(
        conn: &mut DbConnection,
        event_id: EventId,
        per_page: i64,
        page: i64,
        filter_by_status: Option<EventInviteStatus>,
    ) -> Result<(Vec<(EventInvite, User)>, i64)> {
        let allowed_states = filter_by_status
            .map(|s| BTreeSet::from([s]))
            .unwrap_or_else(EventInviteStatus::all_enum_values);

        let query = event_invites::table
            .inner_join(users::table.on(event_invites::invitee.eq(users::id)))
            .filter(
                event_invites::columns::event_id
                    .eq(event_id)
                    .and(event_invites::columns::status.eq_any(allowed_states)),
            )
            .order(event_invites::created_at.desc())
            .then_order_by(event_invites::created_by.desc())
            .then_order_by(event_invites::invitee.desc())
            .paginate_by(per_page, page);

        let invites = query.load_and_count(conn).await?;

        Ok(invites)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn get_pending_for_user(
        conn: &mut DbConnection,
        user_id: UserId,
    ) -> Result<Vec<EventInvite>> {
        let query = event_invites::table.filter(
            event_invites::invitee
                .eq(user_id)
                .and(event_invites::status.eq(EventInviteStatus::Pending)),
        );

        let event_invites = query.load(conn).await?;

        Ok(event_invites)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn get_for_user_and_room(
        conn: &mut DbConnection,
        user_id: UserId,
        room_id: RoomId,
    ) -> Result<Option<EventInvite>> {
        let query = event_invites::table
            .select(event_invites::all_columns)
            .inner_join(
                events::table.on(events::id
                    .eq(event_invites::event_id)
                    .and(events::room.eq(room_id))),
            )
            .filter(event_invites::invitee.eq(user_id));

        let event_invite = query.first(conn).await.optional()?;

        Ok(event_invite)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn delete_by_invitee(
        conn: &mut DbConnection,
        event_id: EventId,
        invitee: UserId,
    ) -> Result<EventInvite> {
        let query = diesel::delete(event_invites::table)
            .filter(
                event_invites::event_id
                    .eq(event_id)
                    .and(event_invites::invitee.eq(invitee)),
            )
            .returning(event_invites::all_columns);

        let event_invite = query.get_result(conn).await?;

        Ok(event_invite)
    }
}

#[derive(Insertable)]
#[diesel(table_name = event_invites)]
pub struct NewEventInvite {
    pub event_id: EventId,
    pub invitee: UserId,
    pub role: InviteRole,
    pub created_by: UserId,
    pub created_at: Option<DateTime<Utc>>,
}

impl NewEventInvite {
    /// Tries to insert the EventInvite into the database
    ///
    /// When yielding a unique key violation, None is returned.
    #[tracing::instrument(err, skip_all)]
    pub async fn try_insert(self, conn: &mut DbConnection) -> Result<Option<EventInvite>> {
        let query = self.insert_into(event_invites::table);

        let result = query.get_result(conn).await;

        match result {
            Ok(event_invite) => Ok(Some(event_invite)),
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                ..,
            )) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = event_invites)]
pub struct UpdateEventInvite {
    pub status: Option<EventInviteStatus>,
    pub role: Option<InviteRole>,
}

impl UpdateEventInvite {
    /// Apply the update to the invite where `user_id` is the invitee
    #[tracing::instrument(err, skip_all)]
    pub async fn apply(
        self,
        conn: &mut DbConnection,
        user_id: UserId,
        event_id: EventId,
    ) -> Result<EventInvite> {
        // TODO: Check if the update actually applied a change (also have a look at fn `apply` of `UpdateEventEmailInvite`)
        // Use something like
        // UPDATE event_invites SET status = $status WHERE id = $id RETURNING id, status, (SELECT status FROM tmp WHERE id = $id);
        // or
        // UPDATE event_invites SET status = $status WHERE id = $id FROM event_invites old RETURNING old.*;
        // and compare the value to the set one to return if the value was changed
        let query = diesel::update(event_invites::table)
            .filter(
                event_invites::event_id
                    .eq(event_id)
                    .and(event_invites::invitee.eq(user_id)),
            )
            .set(self)
            // change it here
            .returning(event_invites::all_columns);

        let event_invite = query.get_result(conn).await?;

        Ok(event_invite)
    }
}

#[derive(Associations, Identifiable, Queryable)]
#[diesel(table_name = event_favorites)]
#[diesel(primary_key(user_id, event_id))]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Event))]
pub struct EventFavorite {
    pub user_id: UserId,
    pub event_id: EventId,
}

impl EventFavorite {
    /// Deletes a EventFavorite entry by user_id and event_id
    ///
    /// Returns true if something was deleted
    #[tracing::instrument(err, skip_all)]
    pub async fn delete_by_id(
        conn: &mut DbConnection,
        user_id: UserId,
        event_id: EventId,
    ) -> Result<bool> {
        let lines_changes = diesel::delete(event_favorites::table)
            .filter(
                event_favorites::user_id
                    .eq(user_id)
                    .and(event_favorites::event_id.eq(event_id)),
            )
            .execute(conn)
            .await?;

        Ok(lines_changes > 0)
    }
}

#[derive(Insertable)]
#[diesel(table_name = event_favorites)]
pub struct NewEventFavorite {
    pub user_id: UserId,
    pub event_id: EventId,
}

impl NewEventFavorite {
    /// Tries to insert the NewEventFavorite into the database
    ///
    /// When yielding a unique key violation, None is returned.
    #[tracing::instrument(err, skip_all)]
    pub async fn try_insert(self, conn: &mut DbConnection) -> Result<Option<EventFavorite>> {
        let query = self.insert_into(event_favorites::table);

        let result = query.get_result(conn).await;

        match result {
            Ok(event_favorite) => Ok(Some(event_favorite)),
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                ..,
            )) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

#[derive(Debug, Insertable, Queryable, Identifiable, Associations)]
#[diesel(table_name = event_training_participation_report_parameter_sets)]
#[diesel(primary_key(event_id))]
#[diesel(belongs_to(Event, foreign_key = event_id))]
pub struct EventTrainingParticipationReportParameterSet {
    pub event_id: EventId,
    pub initial_checkpoint_delay_after: i64,
    pub initial_checkpoint_delay_within: i64,
    pub checkpoint_interval_after: i64,
    pub checkpoint_interval_within: i64,
}

impl EventTrainingParticipationReportParameterSet {
    #[tracing::instrument(err, skip_all)]
    pub async fn get_for_event(conn: &mut DbConnection, event_id: EventId) -> Result<Option<Self>> {
        let parameter_set = event_training_participation_report_parameter_sets::table
            .filter(event_training_participation_report_parameter_sets::event_id.eq(event_id))
            .get_result(conn)
            .await
            .optional()?;

        Ok(parameter_set)
    }

    /// Tries to insert the EventTrainingParticipationParameterSet into the database
    ///
    /// When yielding a unique key violation, None is returned.
    #[tracing::instrument(err, skip_all)]
    pub async fn try_insert(self, conn: &mut DbConnection) -> Result<Option<Self>> {
        let query = self.insert_into(event_training_participation_report_parameter_sets::table);

        let result = query.get_result(conn).await;

        match result {
            Ok(event_invite) => Ok(Some(event_invite)),
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                ..,
            )) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn delete_by_id(conn: &mut DbConnection, event_id: EventId) -> Result<()> {
        let query = diesel::delete(
            event_training_participation_report_parameter_sets::table
                .filter(event_training_participation_report_parameter_sets::event_id.eq(event_id)),
        );

        query.execute(conn).await?;

        Ok(())
    }
}

impl From<EventTrainingParticipationReportParameterSet>
    for TrainingParticipationReportParameterSet
{
    fn from(
        EventTrainingParticipationReportParameterSet {
            event_id: _,
            initial_checkpoint_delay_after,
            initial_checkpoint_delay_within,
            checkpoint_interval_after,
            checkpoint_interval_within,
        }: EventTrainingParticipationReportParameterSet,
    ) -> Self {
        Self {
            initial_checkpoint_delay: TimeRange {
                after: u64::try_from(initial_checkpoint_delay_after).unwrap_or_default(),
                within: u64::try_from(initial_checkpoint_delay_within).unwrap_or_default(),
            },
            checkpoint_interval: TimeRange {
                after: u64::try_from(checkpoint_interval_after).unwrap_or_default(),
                within: u64::try_from(checkpoint_interval_within).unwrap_or_default(),
            },
        }
    }
}

impl From<(EventId, TrainingParticipationReportParameterSet)>
    for EventTrainingParticipationReportParameterSet
{
    fn from(
        (
            event_id,
            TrainingParticipationReportParameterSet {
                initial_checkpoint_delay,
                checkpoint_interval,
            },
        ): (EventId, TrainingParticipationReportParameterSet),
    ) -> Self {
        Self {
            event_id,
            initial_checkpoint_delay_after: i64::try_from(initial_checkpoint_delay.after)
                .unwrap_or(i64::MAX),
            initial_checkpoint_delay_within: i64::try_from(initial_checkpoint_delay.within)
                .unwrap_or(i64::MAX),
            checkpoint_interval_after: i64::try_from(checkpoint_interval.after).unwrap_or(i64::MAX),
            checkpoint_interval_within: i64::try_from(checkpoint_interval.within)
                .unwrap_or(i64::MAX),
        }
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = event_training_participation_report_parameter_sets)]
pub struct UpdateEventTrainingParticipationReportParameterSet {
    pub initial_checkpoint_delay_after: Option<i64>,
    pub initial_checkpoint_delay_within: Option<i64>,
    pub checkpoint_interval_after: Option<i64>,
    pub checkpoint_interval_within: Option<i64>,
}

impl UpdateEventTrainingParticipationReportParameterSet {
    /// Apply the update to the invite where `user_id` is the invitee
    #[tracing::instrument(err, skip_all)]
    pub async fn apply(
        self,
        conn: &mut DbConnection,
        event_id: EventId,
    ) -> Result<EventTrainingParticipationReportParameterSet> {
        let query = diesel::update(event_training_participation_report_parameter_sets::table)
            .filter(event_training_participation_report_parameter_sets::event_id.eq(event_id))
            .set(self)
            // change it here
            .returning(event_training_participation_report_parameter_sets::all_columns);

        let event_training_participation_report_parameter_sets = query.get_result(conn).await?;

        Ok(event_training_participation_report_parameter_sets)
    }
}

impl From<TrainingParticipationReportParameterSet>
    for UpdateEventTrainingParticipationReportParameterSet
{
    fn from(
        TrainingParticipationReportParameterSet {
            initial_checkpoint_delay,
            checkpoint_interval,
        }: TrainingParticipationReportParameterSet,
    ) -> Self {
        Self {
            initial_checkpoint_delay_after: Some(
                i64::try_from(initial_checkpoint_delay.after).unwrap_or(i64::MAX),
            ),
            initial_checkpoint_delay_within: Some(
                i64::try_from(initial_checkpoint_delay.within).unwrap_or(i64::MAX),
            ),
            checkpoint_interval_after: Some(
                i64::try_from(checkpoint_interval.after).unwrap_or(i64::MAX),
            ),
            checkpoint_interval_within: Some(
                i64::try_from(checkpoint_interval.within).unwrap_or(i64::MAX),
            ),
        }
    }
}
