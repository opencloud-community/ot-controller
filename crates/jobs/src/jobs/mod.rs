// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! # Jobs that can be run in the OpenTalk job execution system

mod adhoc_event_cleanup;
mod event_cleanup;
mod invite_cleanup;
mod keycloak_account_sync;
mod room_cleanup;
mod self_check;
mod sync_storage_files;
mod user_cleanup;

pub use adhoc_event_cleanup::AdhocEventCleanup;
pub use event_cleanup::EventCleanup;
pub use invite_cleanup::InviteCleanup;
pub use keycloak_account_sync::KeycloakAccountSync;
pub use room_cleanup::RoomCleanup;
pub use self_check::SelfCheck;
pub use sync_storage_files::SyncStorageFiles;
pub use user_cleanup::UserCleanup;

#[cfg(test)]
mod test_utils {
    use opentalk_database::DbConnection;
    use opentalk_db_storage::{
        events::{Event, NewEvent},
        invites::{Invite, NewInvite},
        rooms::{NewRoom, Room},
        users::User,
    };
    use opentalk_test_util::database::DatabaseContext;

    pub(super) async fn create_events_and_independent_rooms(
        db_ctx: &DatabaseContext,
        event_count: u64,
        independent_room_count: u64,
    ) {
        let user = db_ctx.create_test_user(1, vec![]).await.unwrap();

        let mut conn = db_ctx.db.get_conn().await.unwrap();

        for _ in 0..event_count {
            create_generic_test_event(&mut conn, &user).await;
        }

        for _ in 0..independent_room_count {
            create_generic_test_room(&mut conn, &user).await;
        }
    }

    pub(super) async fn create_generic_test_room(conn: &mut DbConnection, user: &User) -> Room {
        let new_room = NewRoom {
            created_by: user.id,
            password: None,
            waiting_room: false,
            e2e_encryption: false,
            tenant_id: user.tenant_id,
        };

        new_room.insert(conn).await.unwrap()
    }

    pub(super) async fn create_generic_test_event(conn: &mut DbConnection, user: &User) -> Event {
        let room = create_generic_test_room(conn, user).await;

        let new_event = NewEvent {
            title: "TestEvent".parse().expect("valid event title"),
            description: "A normal event, created by a test".into(),
            room: room.id,
            created_by: user.id,
            updated_by: user.id,
            is_time_independent: true,
            is_all_day: None,
            starts_at: None,
            starts_at_tz: None,
            ends_at: None,
            ends_at_tz: None,
            duration_secs: None,
            is_recurring: None,
            recurrence_pattern: None,
            is_adhoc: true,
            tenant_id: user.tenant_id,
            show_meeting_details: true,
        };

        new_event.insert(conn).await.unwrap()
    }

    pub(super) async fn create_generic_test_invite(
        conn: &mut DbConnection,
        inviter: &User,
        updated_by: Option<&User>,
        room: &Room,
    ) -> Invite {
        let new_invite = NewInvite {
            created_by: inviter.id,
            updated_by: updated_by.unwrap_or(inviter).id,
            room: room.id,
            active: true,
            expiration: None,
        };

        new_invite.insert(conn).await.unwrap()
    }
}
