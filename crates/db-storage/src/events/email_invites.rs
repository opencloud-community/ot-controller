// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::{Event, NewEventInvite};
use crate::schema::{event_email_invites, event_invites, events};
use crate::users::User;
use chrono::{DateTime, Utc};
use database::{DbConnection, Paginate, Result};
use diesel::prelude::*;
use diesel::{ExpressionMethods, QueryDsl, Queryable};
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use types::core::{EventId, InviteRole, RoomId, UserId};

#[derive(Insertable)]
#[diesel(table_name = event_email_invites)]
pub struct NewEventEmailInvite {
    pub event_id: EventId,
    pub email: String,
    pub created_by: UserId,
}

impl NewEventEmailInvite {
    /// Tries to insert the EventEmailInvite into the database
    ///
    /// When yielding a unique key violation, None is returned.
    #[tracing::instrument(err, skip_all)]
    pub async fn try_insert(self, conn: &mut DbConnection) -> Result<Option<EventEmailInvite>> {
        let query = self.insert_into(event_email_invites::table);

        let result = query.get_result(conn).await;

        match result {
            Ok(event_email_invites) => Ok(Some(event_email_invites)),
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                ..,
            )) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

#[derive(Debug, Associations, Identifiable, Queryable)]
#[diesel(table_name = event_email_invites)]
#[diesel(primary_key(event_id, email))]
#[diesel(belongs_to(Event))]
pub struct EventEmailInvite {
    pub event_id: EventId,
    pub email: String,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
}

impl EventEmailInvite {
    pub async fn migrate_to_user_invites(
        conn: &mut DbConnection,
        user: &User,
    ) -> Result<Vec<(EventId, RoomId)>> {
        conn.transaction(|conn| {
            async move {
                let email_invites_with_room: Vec<(EventEmailInvite, RoomId)> =
                    event_email_invites::table
                        .filter(event_email_invites::email.eq(&user.email))
                        .inner_join(events::table)
                        .filter(events::tenant_id.eq(user.tenant_id))
                        .select((event_email_invites::all_columns, events::room))
                        .load(conn)
                        .await?;

                if email_invites_with_room.is_empty() {
                    return Ok(vec![]);
                }

                let event_ids = email_invites_with_room
                    .iter()
                    .map(|(email_invite, room_id)| (email_invite.event_id, *room_id))
                    .collect();

                let new_invites: Vec<_> = email_invites_with_room
                    .into_iter()
                    .map(|(email_invite, _)| NewEventInvite {
                        event_id: email_invite.event_id,
                        invitee: user.id,
                        role: InviteRole::User,
                        created_by: email_invite.created_by,
                        created_at: Some(email_invite.created_at),
                    })
                    .collect();

                diesel::insert_into(event_invites::table)
                    .values(new_invites)
                    .on_conflict_do_nothing()
                    .execute(conn)
                    .await?;

                diesel::delete(
                    event_email_invites::table.filter(
                        event_email_invites::email.eq(&user.email).and(
                            event_email_invites::event_id.eq_any(
                                events::table
                                    .filter(events::tenant_id.eq(user.tenant_id))
                                    .select(events::id),
                            ),
                        ),
                    ),
                )
                .execute(conn)
                .await?;

                Ok(event_ids)
            }
            .scope_boxed()
        })
        .await
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn get_for_events(
        conn: &mut DbConnection,
        events: &[&Event],
    ) -> Result<Vec<Vec<EventEmailInvite>>> {
        let invites: Vec<EventEmailInvite> =
            EventEmailInvite::belonging_to(events).load(conn).await?;

        let invites_by_event: Vec<Vec<EventEmailInvite>> = invites.grouped_by(events);
        Ok(invites_by_event)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn delete(
        conn: &mut DbConnection,
        event_id: &EventId,
        email: &String,
    ) -> Result<EventEmailInvite> {
        let query = diesel::delete(event_email_invites::table)
            .filter(
                event_email_invites::event_id
                    .eq(event_id)
                    .and(event_email_invites::email.eq(email)),
            )
            .returning(event_email_invites::all_columns);

        let invite = query.get_result(conn).await?;

        Ok(invite)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn get_for_event_paginated(
        conn: &mut DbConnection,
        event_id: EventId,
        limit: i64,
        page: i64,
    ) -> Result<(Vec<EventEmailInvite>, i64)> {
        let query = event_email_invites::table
            .filter(event_email_invites::columns::event_id.eq(event_id))
            .order(event_email_invites::created_at.desc())
            .then_order_by(event_email_invites::created_by.desc())
            .then_order_by(event_email_invites::email.desc())
            .paginate_by(limit, page);

        let invites: (Vec<EventEmailInvite>, i64) = query.load_and_count(conn).await?;

        Ok(invites)
    }
}
