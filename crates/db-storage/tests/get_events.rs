// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::common::make_user;
use chrono::{TimeZone as _, Utc};
use chrono_tz::Tz;
use database::DbConnection;
use opentalk_db_storage::events::{
    Event, EventInvite, GetEventsCursor, NewEvent, NewEventInvite, UpdateEventInvite,
};
use opentalk_db_storage::rooms::NewRoom;
use opentalk_db_storage::tenants::{get_or_create_tenant_by_oidc_id, OidcTenantId};
use pretty_assertions::assert_eq;
use serial_test::serial;
use types::core::{EventId, EventInviteStatus, InviteRole, RoomId, TimeZone, UserId};

mod common;

async fn make_event(
    conn: &mut DbConnection,
    user_id: UserId,
    room_id: RoomId,
    hour: Option<u32>,
    is_adhoc: bool,
) -> Event {
    let tenant = get_or_create_tenant_by_oidc_id(conn, &OidcTenantId::from("default".to_string()))
        .await
        .unwrap();
    NewEvent {
        title: "Test Event".into(),
        description: "Test Event".into(),
        room: room_id,
        created_by: user_id,
        updated_by: user_id,
        is_time_independent: hour.is_none(),
        is_all_day: Some(false),
        starts_at: hour.map(|h| Tz::UTC.with_ymd_and_hms(2020, 1, 1, h, 0, 0).unwrap()),
        starts_at_tz: hour.map(|_| TimeZone::from(Tz::UTC)),
        ends_at: hour.map(|h| Tz::UTC.with_ymd_and_hms(2020, 1, 1, h, 0, 0).unwrap()),
        ends_at_tz: hour.map(|_| TimeZone::from(Tz::UTC)),
        duration_secs: hour.map(|_| 0),
        is_recurring: Some(false),
        recurrence_pattern: None,
        is_adhoc,
        tenant_id: tenant.id,
    }
    .insert(conn)
    .await
    .unwrap()
}

async fn update_invite_status(
    conn: &mut DbConnection,
    user_id: UserId,
    event_id: EventId,
    new_status: EventInviteStatus,
) {
    let changeset = UpdateEventInvite {
        status: Some(new_status),
        role: None,
    };

    changeset.apply(conn, user_id, event_id).await.unwrap();
}

#[tokio::test]
#[serial]
async fn test() {
    let db_ctx = test_util::database::DatabaseContext::new(true).await;

    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let user = make_user(&mut conn, "Test", "Tester", "Test Tester").await;

    let room = NewRoom {
        created_by: user.id,
        password: None,
        waiting_room: false,
        tenant_id: user.tenant_id,
    }
    .insert(&mut conn)
    .await
    .unwrap();

    // create events. The variable number indicates its expected ordering

    // first two events, first on on hour 2 then 1.
    // This tests the ordering of comparison of times (e.g. starts_at, then created_at)
    let event2 = make_event(&mut conn, user.id, room.id, Some(2), true).await;
    let event1 = make_event(&mut conn, user.id, room.id, Some(1), true).await;

    // this event should come last because starts_at is largest
    let event8 = make_event(&mut conn, user.id, room.id, Some(10), true).await;

    // Test that created_at is being honored if starts_at is equal
    let event3 = make_event(&mut conn, user.id, room.id, Some(3), true).await;
    let event4 = make_event(&mut conn, user.id, room.id, Some(3), true).await;
    let event5 = make_event(&mut conn, user.id, room.id, Some(3), true).await;
    let event6 = make_event(&mut conn, user.id, room.id, Some(3), true).await;
    let event7 = make_event(&mut conn, user.id, room.id, Some(3), true).await;

    {
        // Test cursor

        // Get first two events 1, 2
        let first_two = Event::get_all_for_user_paginated(
            &mut conn,
            &user,
            false,
            vec![],
            None,
            None,
            None,
            None,
            None,
            2,
        )
        .await
        .unwrap();
        assert_eq!(first_two.len(), 2);
        let query_event1 = &first_two[0].0;
        let query_event2 = &first_two[1].0;
        assert_eq!(query_event1, &event1);
        assert_eq!(query_event2, &event2);

        // Make cursor from last event fetched
        let cursor = GetEventsCursor::from_last_event_in_query(query_event2);

        // Use that to get 3,4
        let next_two = Event::get_all_for_user_paginated(
            &mut conn,
            &user,
            false,
            vec![],
            None,
            None,
            None,
            None,
            Some(cursor),
            2,
        )
        .await
        .unwrap();
        assert_eq!(first_two.len(), 2);
        let query_event3 = &next_two[0].0;
        let query_event4 = &next_two[1].0;
        assert_eq!(query_event3, &event3);
        assert_eq!(query_event4, &event4);

        // Then 5,6
        let cursor = GetEventsCursor::from_last_event_in_query(query_event4);

        let next_two = Event::get_all_for_user_paginated(
            &mut conn,
            &user,
            false,
            vec![],
            None,
            None,
            None,
            None,
            Some(cursor),
            2,
        )
        .await
        .unwrap();
        assert_eq!(first_two.len(), 2);
        let query_event5 = &next_two[0].0;
        let query_event6 = &next_two[1].0;
        assert_eq!(query_event5, &event5);
        assert_eq!(query_event6, &event6);

        // Then 7,8
        let cursor = GetEventsCursor::from_last_event_in_query(query_event6);

        let next_two = Event::get_all_for_user_paginated(
            &mut conn,
            &user,
            false,
            vec![],
            None,
            None,
            None,
            None,
            Some(cursor),
            2,
        )
        .await
        .unwrap();
        assert_eq!(first_two.len(), 2);
        let query_event7 = &next_two[0].0;
        let query_event8 = &next_two[1].0;
        assert_eq!(query_event7, &event7);
        assert_eq!(query_event8, &event8);
    }

    {
        // Test time_min
        let only_event8 = Event::get_all_for_user_paginated(
            &mut conn,
            &user,
            false,
            vec![],
            Some(Utc.with_ymd_and_hms(2020, 1, 1, 5, 0, 0).unwrap()),
            None,
            None,
            None,
            None,
            100,
        )
        .await
        .unwrap();
        assert_eq!(only_event8.len(), 1);
        assert_eq!(only_event8[0].0, event8);
    }

    {
        // Test time_max
        let every_event_except_event8 = Event::get_all_for_user_paginated(
            &mut conn,
            &user,
            false,
            vec![],
            None,
            Some(Utc.with_ymd_and_hms(2020, 1, 1, 5, 0, 0).unwrap()),
            None,
            None,
            None,
            100,
        )
        .await
        .unwrap();
        assert_eq!(every_event_except_event8.len(), 7);
        assert_eq!(every_event_except_event8[0].0, event1);
        assert_eq!(every_event_except_event8[1].0, event2);
        assert_eq!(every_event_except_event8[2].0, event3);
        assert_eq!(every_event_except_event8[3].0, event4);
        assert_eq!(every_event_except_event8[4].0, event5);
        assert_eq!(every_event_except_event8[5].0, event6);
        assert_eq!(every_event_except_event8[6].0, event7);
        assert_eq!(every_event_except_event8[0].0, event1);
    }
    {
        // Test both time_min + time_max
        let only_event_at_3h = Event::get_all_for_user_paginated(
            &mut conn,
            &user,
            false,
            vec![],
            Some(Utc.with_ymd_and_hms(2020, 1, 1, 3, 0, 0).unwrap()),
            Some(Utc.with_ymd_and_hms(2020, 1, 1, 3, 0, 0).unwrap()),
            None,
            None,
            None,
            100,
        )
        .await
        .unwrap();
        assert_eq!(only_event_at_3h.len(), 5);
        assert_eq!(only_event_at_3h[0].0, event3);
        assert_eq!(only_event_at_3h[1].0, event4);
        assert_eq!(only_event_at_3h[2].0, event5);
        assert_eq!(only_event_at_3h[3].0, event6);
        assert_eq!(only_event_at_3h[4].0, event7);
    }
}

#[tokio::test]
#[serial]
async fn get_events_invite_filter() {
    let db_ctx = test_util::database::DatabaseContext::new(true).await;

    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let inviter = make_user(&mut conn, "Ingo", "Inviter", "inviter").await;
    let invitee = make_user(&mut conn, "Ingrid", "Invitee", "invitee").await;

    let room = NewRoom {
        created_by: inviter.id,
        password: None,
        waiting_room: false,
        tenant_id: inviter.tenant_id,
    }
    .insert(&mut conn)
    .await
    .unwrap();

    let accept_event = make_event(&mut conn, inviter.id, room.id, Some(1), true).await;
    let decline_event = make_event(&mut conn, inviter.id, room.id, Some(1), true).await;
    let tentative_event = make_event(&mut conn, inviter.id, room.id, Some(1), true).await;
    let pending_event = make_event(&mut conn, inviter.id, room.id, Some(1), true).await;

    // Check that the creator of the events gets created events when filtering by `Accepted` invite status
    let all_events = Event::get_all_for_user_paginated(
        &mut conn,
        &inviter,
        false,
        vec![EventInviteStatus::Accepted],
        None,
        None,
        None,
        None,
        None,
        100,
    )
    .await
    .unwrap();

    assert_eq!(all_events.len(), 4);
    assert!(all_events
        .iter()
        .any(|(event, ..)| event.id == accept_event.id));
    assert!(all_events
        .iter()
        .any(|(event, ..)| event.id == decline_event.id));
    assert!(all_events
        .iter()
        .any(|(event, ..)| event.id == tentative_event.id));
    assert!(all_events
        .iter()
        .any(|(event, ..)| event.id == pending_event.id));

    // Check that no events are returned when filtering for `Declined`
    let no_events = Event::get_all_for_user_paginated(
        &mut conn,
        &inviter,
        false,
        vec![EventInviteStatus::Declined],
        None,
        None,
        None,
        None,
        None,
        100,
    )
    .await
    .unwrap();

    assert!(no_events.is_empty());

    let events = vec![
        &accept_event,
        &decline_event,
        &tentative_event,
        &pending_event,
    ];

    // invite the invitee to all events
    for event in events {
        NewEventInvite {
            event_id: event.id,
            invitee: invitee.id,
            created_by: inviter.id,
            created_at: None,
            role: InviteRole::User,
        }
        .try_insert(&mut conn)
        .await
        .unwrap();
    }

    update_invite_status(
        &mut conn,
        invitee.id,
        accept_event.id,
        EventInviteStatus::Accepted,
    )
    .await;

    update_invite_status(
        &mut conn,
        invitee.id,
        decline_event.id,
        EventInviteStatus::Declined,
    )
    .await;

    update_invite_status(
        &mut conn,
        invitee.id,
        tentative_event.id,
        EventInviteStatus::Tentative,
    )
    .await;

    // check `accepted` invites
    let accepted_events = Event::get_all_for_user_paginated(
        &mut conn,
        &invitee,
        false,
        vec![EventInviteStatus::Accepted],
        None,
        None,
        None,
        None,
        None,
        100,
    )
    .await
    .unwrap();

    assert_eq!(accepted_events.len(), 1);
    assert!(accepted_events
        .iter()
        .any(|(event, ..)| event.id == accept_event.id));

    // check `declined` invites
    let declined_events = Event::get_all_for_user_paginated(
        &mut conn,
        &invitee,
        false,
        vec![EventInviteStatus::Declined],
        None,
        None,
        None,
        None,
        None,
        100,
    )
    .await
    .unwrap();

    assert_eq!(declined_events.len(), 1);
    assert!(declined_events
        .iter()
        .any(|(event, ..)| event.id == decline_event.id));

    // check `tentative` invites
    let tentative_events = Event::get_all_for_user_paginated(
        &mut conn,
        &invitee,
        false,
        vec![EventInviteStatus::Tentative],
        None,
        None,
        None,
        None,
        None,
        100,
    )
    .await
    .unwrap();

    assert_eq!(tentative_events.len(), 1);
    assert!(tentative_events
        .iter()
        .any(|(event, ..)| event.id == tentative_event.id));

    // check `pending` invites
    let pending_events = Event::get_all_for_user_paginated(
        &mut conn,
        &invitee,
        false,
        vec![EventInviteStatus::Pending],
        None,
        None,
        None,
        None,
        None,
        100,
    )
    .await
    .unwrap();

    assert_eq!(pending_events.len(), 1);
    assert!(pending_events
        .iter()
        .any(|(event, ..)| event.id == pending_event.id));

    // expect all events when no invite_status_filter is set
    let all_events = Event::get_all_for_user_paginated(
        &mut conn,
        &invitee,
        false,
        vec![],
        None,
        None,
        None,
        None,
        None,
        100,
    )
    .await
    .unwrap();

    assert_eq!(all_events.len(), 4);
    assert!(all_events
        .iter()
        .any(|(event, ..)| event.id == accept_event.id));
    assert!(all_events
        .iter()
        .any(|(event, ..)| event.id == decline_event.id));
    assert!(all_events
        .iter()
        .any(|(event, ..)| event.id == tentative_event.id));
    assert!(all_events
        .iter()
        .any(|(event, ..)| event.id == pending_event.id));
}

#[tokio::test]
#[serial]
async fn get_event_invites() {
    let db_ctx = test_util::database::DatabaseContext::new(true).await;

    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let ferdinand = make_user(&mut conn, "Ferdinand", "Jaegermeister", "ferdemeister").await;
    let louise = make_user(&mut conn, "Jeez", "Louise", "Jesus").await;
    let gerhard = make_user(&mut conn, "Gerhard", "Bauer", "Hardi").await;

    let room = NewRoom {
        created_by: ferdinand.id,
        password: None,
        waiting_room: false,
        tenant_id: ferdinand.tenant_id,
    }
    .insert(&mut conn)
    .await
    .unwrap();

    // EVENT 1 MIT JEEZ LOUISE AND GERHARD
    let event1 = make_event(&mut conn, ferdinand.id, room.id, Some(1), true).await;

    NewEventInvite {
        event_id: event1.id,
        invitee: louise.id,
        created_by: ferdinand.id,
        created_at: None,
        role: InviteRole::User,
    }
    .try_insert(&mut conn)
    .await
    .unwrap();

    NewEventInvite {
        event_id: event1.id,
        invitee: gerhard.id,
        created_by: ferdinand.id,
        created_at: None,
        role: InviteRole::Moderator,
    }
    .try_insert(&mut conn)
    .await
    .unwrap();

    // EVENT 2 MIT JEEZ LOUSE UND FERDINAND
    let event2 = make_event(&mut conn, gerhard.id, room.id, Some(1), true).await;

    NewEventInvite {
        event_id: event2.id,
        invitee: louise.id,
        created_by: gerhard.id,
        created_at: None,
        role: InviteRole::User,
    }
    .try_insert(&mut conn)
    .await
    .unwrap();

    NewEventInvite {
        event_id: event2.id,
        invitee: ferdinand.id,
        created_by: gerhard.id,
        created_at: None,
        role: InviteRole::Moderator,
    }
    .try_insert(&mut conn)
    .await
    .unwrap();

    let events = &[&event1, &event2][..];
    let invites_with_invitees = EventInvite::get_for_events(&mut conn, events)
        .await
        .unwrap();

    for (event, invites_with_users) in events.iter().zip(invites_with_invitees) {
        println!("Event: {event:#?}");
        println!(
            "Invitees: {:#?}",
            invites_with_users
                .into_iter()
                .map(|x| x.1)
                .collect::<Vec<_>>()
        );
        println!("#################################################")
    }
}

#[tokio::test]
#[serial]
async fn get_event_adhoc() {
    let db_ctx = test_util::database::DatabaseContext::new(true).await;

    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let user = make_user(&mut conn, "Test", "Tester", "Test Tester").await;

    let room = NewRoom {
        created_by: user.id,
        password: None,
        waiting_room: false,
        tenant_id: user.tenant_id,
    }
    .insert(&mut conn)
    .await
    .unwrap();

    let event1 = make_event(&mut conn, user.id, room.id, Some(1), true).await;
    let event2 = make_event(&mut conn, user.id, room.id, Some(1), false).await;
    let event3 = make_event(&mut conn, user.id, room.id, Some(1), false).await;
    let event4 = make_event(&mut conn, user.id, room.id, Some(1), true).await;
    let event5 = make_event(&mut conn, user.id, room.id, Some(1), true).await;

    let all = Event::get_all_for_user_paginated(
        &mut conn,
        &user,
        false,
        vec![],
        None,
        None,
        None,
        None,
        None,
        10,
    )
    .await
    .unwrap();

    assert_eq!(all.len(), 5);
    assert_eq!(all[0].0, event1);
    assert_eq!(all[1].0, event2);
    assert_eq!(all[2].0, event3);
    assert_eq!(all[3].0, event4);
    assert_eq!(all[4].0, event5);

    let adhoc = Event::get_all_for_user_paginated(
        &mut conn,
        &user,
        false,
        vec![],
        None,
        None,
        Some(true),
        None,
        None,
        10,
    )
    .await
    .unwrap();
    assert_eq!(adhoc.len(), 3);
    assert_eq!(adhoc[0].0, event1);
    assert_eq!(adhoc[1].0, event4);
    assert_eq!(adhoc[2].0, event5);

    let non_adhoc = Event::get_all_for_user_paginated(
        &mut conn,
        &user,
        false,
        vec![],
        None,
        None,
        Some(false),
        None,
        None,
        10,
    )
    .await
    .unwrap();
    assert_eq!(non_adhoc.len(), 2);
    assert_eq!(non_adhoc[0].0, event2);
    assert_eq!(non_adhoc[1].0, event3);
}

#[tokio::test]
#[serial]
async fn get_event_time_independent() {
    let db_ctx = test_util::database::DatabaseContext::new(true).await;

    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let user = make_user(&mut conn, "Test", "Tester", "Test Tester").await;

    let room = NewRoom {
        created_by: user.id,
        password: None,
        waiting_room: false,
        tenant_id: user.tenant_id,
    }
    .insert(&mut conn)
    .await
    .unwrap();

    let event1 = make_event(&mut conn, user.id, room.id, None, false).await;
    let event2 = make_event(&mut conn, user.id, room.id, None, false).await;
    let event3 = make_event(&mut conn, user.id, room.id, Some(1), false).await;
    let event4 = make_event(&mut conn, user.id, room.id, None, false).await;
    let event5 = make_event(&mut conn, user.id, room.id, Some(2), false).await;

    let all = Event::get_all_for_user_paginated(
        &mut conn,
        &user,
        false,
        vec![],
        None,
        None,
        None,
        None,
        None,
        10,
    )
    .await
    .unwrap();

    // different order than creation order, because events without
    // starts_at field are sorted first
    assert_eq!(all.len(), 5);
    assert_eq!(all[0].0, event1);
    assert_eq!(all[1].0, event2);
    assert_eq!(all[2].0, event4);
    assert_eq!(all[3].0, event3);
    assert_eq!(all[4].0, event5);

    let time_independent = Event::get_all_for_user_paginated(
        &mut conn,
        &user,
        false,
        vec![],
        None,
        None,
        None,
        Some(true),
        None,
        10,
    )
    .await
    .unwrap();
    assert_eq!(time_independent.len(), 3);
    assert_eq!(time_independent[0].0, event1);
    assert_eq!(time_independent[1].0, event2);
    assert_eq!(time_independent[2].0, event4);

    let time_dependent = Event::get_all_for_user_paginated(
        &mut conn,
        &user,
        false,
        vec![],
        None,
        None,
        None,
        Some(false),
        None,
        10,
    )
    .await
    .unwrap();
    assert_eq!(time_dependent.len(), 2);
    assert_eq!(time_dependent[0].0, event3);
    assert_eq!(time_dependent[1].0, event5);
}

#[tokio::test]
#[serial]
async fn get_event_min_max_time() {
    let db_ctx = test_util::database::DatabaseContext::new(true).await;

    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let user = make_user(&mut conn, "Test", "Tester", "Test Tester").await;

    let room = NewRoom {
        created_by: user.id,
        password: None,
        waiting_room: false,
        tenant_id: user.tenant_id,
    }
    .insert(&mut conn)
    .await
    .unwrap();

    let event1 = NewEvent {
        title: "Test Event".into(),
        description: "Test Event".into(),
        room: room.id,
        created_by: user.id,
        updated_by: user.id,
        is_time_independent: false,
        is_all_day: Some(false),
        starts_at: None,
        starts_at_tz: None,
        ends_at: None,
        ends_at_tz: None,
        duration_secs: None,
        is_recurring: Some(false),
        recurrence_pattern: None,
        is_adhoc: false,
        tenant_id: user.tenant_id,
    }
    .insert(&mut conn)
    .await
    .unwrap();

    let event2 = NewEvent {
        title: "Test Event".into(),
        description: "Test Event".into(),
        room: room.id,
        created_by: user.id,
        updated_by: user.id,
        is_time_independent: false,
        is_all_day: Some(false),
        starts_at: Some(Tz::UTC.with_ymd_and_hms(2020, 1, 1, 10, 0, 0).unwrap()),
        starts_at_tz: Some(TimeZone::from(Tz::UTC)),
        ends_at: Some(Tz::UTC.with_ymd_and_hms(2020, 1, 1, 11, 0, 0).unwrap()),
        ends_at_tz: Some(TimeZone::from(Tz::UTC)),
        duration_secs: Some(3600),
        is_recurring: Some(false),
        recurrence_pattern: None,
        is_adhoc: false,
        tenant_id: user.tenant_id,
    }
    .insert(&mut conn)
    .await
    .unwrap();

    {
        // Query without any time restrictions
        let events = Event::get_all_for_user_paginated(
            &mut conn,
            &user,
            false,
            vec![],
            None,
            None,
            None,
            None,
            None,
            10,
        )
        .await
        .unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].0, event1);
        assert_eq!(events[1].0, event2);
    }

    {
        // Query an open timeframe before the event
        let events = Event::get_all_for_user_paginated(
            &mut conn,
            &user,
            false,
            vec![],
            None,
            Some(Utc.with_ymd_and_hms(2020, 1, 1, 9, 0, 0).unwrap()),
            None,
            None,
            None,
            10,
        )
        .await
        .unwrap();
        assert!(events.is_empty());
    }

    {
        // Query a closed timeframe before the event
        let events = Event::get_all_for_user_paginated(
            &mut conn,
            &user,
            false,
            vec![],
            Some(Utc.with_ymd_and_hms(2020, 1, 1, 8, 0, 0).unwrap()),
            Some(Utc.with_ymd_and_hms(2020, 1, 1, 9, 0, 0).unwrap()),
            None,
            None,
            None,
            10,
        )
        .await
        .unwrap();
        assert!(events.is_empty());
    }

    {
        // Query an open timeframe after the event
        let events = Event::get_all_for_user_paginated(
            &mut conn,
            &user,
            false,
            vec![],
            Some(Utc.with_ymd_and_hms(2020, 1, 1, 12, 0, 0).unwrap()),
            None,
            None,
            None,
            None,
            10,
        )
        .await
        .unwrap();
        assert!(events.is_empty());
    }

    {
        // Query an closed timeframe after the event
        let events = Event::get_all_for_user_paginated(
            &mut conn,
            &user,
            false,
            vec![],
            Some(Utc.with_ymd_and_hms(2020, 1, 1, 12, 0, 0).unwrap()),
            Some(Utc.with_ymd_and_hms(2020, 1, 1, 13, 0, 0).unwrap()),
            None,
            None,
            None,
            10,
        )
        .await
        .unwrap();
        assert!(events.is_empty());
    }

    {
        // Query a timeframe ending at the start of the event
        let events = Event::get_all_for_user_paginated(
            &mut conn,
            &user,
            false,
            vec![],
            None,
            Some(Utc.with_ymd_and_hms(2020, 1, 1, 10, 0, 0).unwrap()),
            None,
            None,
            None,
            10,
        )
        .await
        .unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, event2);
    }

    {
        // Query a timeframe starting at the end of the event
        let events = Event::get_all_for_user_paginated(
            &mut conn,
            &user,
            false,
            vec![],
            Some(Utc.with_ymd_and_hms(2020, 1, 1, 11, 0, 0).unwrap()),
            None,
            None,
            None,
            None,
            10,
        )
        .await
        .unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, event2);
    }

    {
        // Query an open timeframe overlapping the first half of the event
        let events = Event::get_all_for_user_paginated(
            &mut conn,
            &user,
            false,
            vec![],
            None,
            Some(Utc.with_ymd_and_hms(2020, 1, 1, 10, 30, 0).unwrap()),
            None,
            None,
            None,
            10,
        )
        .await
        .unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, event2);
    }

    {
        // Query a timeframe overlapping the first half of the event
        let events = Event::get_all_for_user_paginated(
            &mut conn,
            &user,
            false,
            vec![],
            Some(Utc.with_ymd_and_hms(2020, 1, 1, 9, 30, 0).unwrap()),
            Some(Utc.with_ymd_and_hms(2020, 1, 1, 10, 30, 0).unwrap()),
            None,
            None,
            None,
            10,
        )
        .await
        .unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, event2);
    }

    {
        // Query an open timeframe overlapping the second half of the event
        let events = Event::get_all_for_user_paginated(
            &mut conn,
            &user,
            false,
            vec![],
            Some(Utc.with_ymd_and_hms(2020, 1, 1, 10, 30, 0).unwrap()),
            None,
            None,
            None,
            None,
            10,
        )
        .await
        .unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, event2);
    }

    {
        // Query a timeframe overlapping the second half of the event
        let events = Event::get_all_for_user_paginated(
            &mut conn,
            &user,
            false,
            vec![],
            Some(Utc.with_ymd_and_hms(2020, 1, 1, 10, 30, 0).unwrap()),
            Some(Utc.with_ymd_and_hms(2020, 1, 1, 11, 30, 0).unwrap()),
            None,
            None,
            None,
            10,
        )
        .await
        .unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, event2);
    }

    {
        // Query a timeframe fully inside the event
        let events = Event::get_all_for_user_paginated(
            &mut conn,
            &user,
            false,
            vec![],
            Some(Utc.with_ymd_and_hms(2020, 1, 1, 10, 20, 0).unwrap()),
            Some(Utc.with_ymd_and_hms(2020, 1, 1, 10, 40, 0).unwrap()),
            None,
            None,
            None,
            10,
        )
        .await
        .unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, event2);
    }

    {
        // Query a timeframe surrounding the event
        let events = Event::get_all_for_user_paginated(
            &mut conn,
            &user,
            false,
            vec![],
            Some(Utc.with_ymd_and_hms(2020, 1, 1, 9, 0, 0).unwrap()),
            Some(Utc.with_ymd_and_hms(2020, 1, 1, 12, 0, 0).unwrap()),
            None,
            None,
            None,
            10,
        )
        .await
        .unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, event2);
    }
}
