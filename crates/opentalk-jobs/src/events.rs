// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    sync::Arc,
};

use chrono::{DateTime, Utc};
use kustos::Authz;
use log::Log;
use opentalk_controller_settings::Settings;
use opentalk_controller_utils::{
    deletion::{Deleter, EventDeleter, RoomDeleter},
    event::EventExt as _,
};
use opentalk_database::{Db, DbConnection};
use opentalk_db_storage::{
    events::Event,
    users::{UpdateUser, User},
};
use opentalk_log::{debug, info, warn};
use opentalk_signaling_core::{ExchangeHandle, ObjectStorage};
use opentalk_types_common::{events::EventId, rooms::RoomId, users::UserId};
use snafu::Report;

use crate::Error;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum DeleteSelector {
    AdHocCreatedBefore(DateTime<Utc>),
    ScheduledThatEndedBefore(DateTime<Utc>),
    BelongingToUser(UserId),
}

pub(crate) async fn perform_deletion(
    logger: &dyn Log,
    db: Arc<Db>,
    exchange_handle: ExchangeHandle,
    settings: &Settings,
    fail_on_shared_folder_deletion_error: bool,
    delete_selector: DeleteSelector,
) -> Result<(), Error> {
    let authz = Authz::new(db.clone()).await?;
    let mut conn = db.get_conn().await?;
    let object_storage = ObjectStorage::new(&settings.minio).await?;

    let orphaned_rooms = delete_events(
        logger,
        &mut conn,
        &authz,
        exchange_handle.clone(),
        settings,
        &object_storage,
        fail_on_shared_folder_deletion_error,
        delete_selector,
    )
    .await?;

    delete_orphaned_rooms(
        logger,
        &mut conn,
        &authz,
        exchange_handle,
        settings,
        &object_storage,
        orphaned_rooms,
        fail_on_shared_folder_deletion_error,
    )
    .await?;

    Ok(())
}

/// Identify and delete events according to the specified delete selector
///
/// Returns the rooms which are orphaned as a result of the event deletion, so
/// they can be cleaned up afterwards
#[allow(clippy::too_many_arguments)]
async fn delete_events(
    logger: &dyn Log,
    conn: &mut DbConnection,
    authz: &Authz,
    exchange_handle: ExchangeHandle,
    settings: &Settings,
    object_storage: &ObjectStorage,
    fail_on_shared_folder_deletion_error: bool,
    delete_selector: DeleteSelector,
) -> Result<HashSet<RoomId>, Error> {
    info!(log: logger, "");
    debug!(log: logger, "Retrieving list of events that should be deleted");

    let candidates = retrieve_deletion_candidate_events(logger, conn, delete_selector).await?;

    let orphaned_rooms = delete_event_candidates(
        logger,
        conn,
        authz,
        exchange_handle,
        settings,
        object_storage,
        fail_on_shared_folder_deletion_error,
        candidates,
    )
    .await;

    Ok(orphaned_rooms)
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn delete_event_candidates(
    logger: &dyn Log,
    conn: &mut DbConnection,
    authz: &Authz,
    exchange_handle: ExchangeHandle,
    settings: &Settings,
    object_storage: &ObjectStorage,
    fail_on_shared_folder_deletion_error: bool,
    candidates: Vec<(EventId, RoomId)>,
) -> HashSet<RoomId> {
    let candidate_count = candidates.len();

    info!(log: logger, "Identified {candidate_count} events for deletion");

    let mut orphaned_rooms = HashSet::new();

    let mut deleter_failures = 0usize;
    for (event_id, room_id) in candidates {
        info!(log: logger, "Deleting event {event_id}");
        let deleter = EventDeleter::new(event_id, fail_on_shared_folder_deletion_error);

        if let Err(e) = deleter
            .perform(
                logger,
                conn,
                authz,
                None,
                exchange_handle.clone(),
                settings,
                object_storage,
            )
            .await
        {
            warn!(log: logger, "Failed deletion: {}", Report::from_error(e));
            deleter_failures += 1;
            continue;
        }

        match Event::get_for_room(conn, room_id).await {
            Ok(Some(_)) => {}
            Ok(None) => {
                let _ = orphaned_rooms.insert(room_id);
            }
            Err(e) => {
                warn!(log: logger, "Failed to retrieve events connected to room: {}", Report::from_error(e));
            }
        }
    }

    info!(log: logger, "Deleted {} events", candidate_count);
    if deleter_failures > 0 {
        warn!(log: logger, "{deleter_failures} events could not be deleted due to errors");
    }
    orphaned_rooms
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn delete_orphaned_rooms(
    logger: &dyn Log,
    conn: &mut DbConnection,
    authz: &Authz,
    exchange_handle: ExchangeHandle,
    settings: &Settings,
    object_storage: &ObjectStorage,
    orphaned_rooms: HashSet<RoomId>,
    fail_on_shared_folder_deletion_error: bool,
) -> Result<(), Error> {
    info!(log: logger, "");
    info!(log: logger, "Deleting orphaned rooms");

    let room_count = orphaned_rooms.len();
    info!(log: logger, "Identified {room_count} orphaned rooms for deletion");

    let mut deleter_failures = 0usize;
    for room_id in orphaned_rooms {
        info!(log: logger, "Deleting room {room_id}");
        let deleter = RoomDeleter::new(room_id, fail_on_shared_folder_deletion_error);

        if let Err(e) = deleter
            .perform(
                logger,
                conn,
                authz,
                None,
                exchange_handle.clone(),
                settings,
                object_storage,
            )
            .await
        {
            warn!(log: logger, "Failed deletion: {}", Report::from_error(e));
            deleter_failures += 1;
        }
    }
    info!(log: logger, "Deleted {} orphaned rooms", room_count);
    if deleter_failures > 0 {
        warn!(log: logger, "{deleter_failures} orphaned rooms could not be deleted due to errors");
    }
    Ok(())
}

pub(crate) async fn retrieve_deletion_candidate_events(
    logger: &dyn Log,
    conn: &mut DbConnection,
    delete_selector: DeleteSelector,
) -> Result<Vec<(EventId, RoomId)>, Error> {
    let events = match delete_selector {
        DeleteSelector::AdHocCreatedBefore(delete_before) => {
            Event::get_all_adhoc_created_before_including_rooms(conn, delete_before).await?
        }
        DeleteSelector::ScheduledThatEndedBefore(delete_before) => {
            get_scheduled_events_that_ended_before(logger, conn, delete_before).await?
        }
        DeleteSelector::BelongingToUser(user_id) => {
            Event::get_all_for_creator_including_rooms(conn, user_id).await?
        }
    };

    Ok(events)
}

async fn get_scheduled_events_that_ended_before(
    logger: &dyn Log,
    conn: &mut DbConnection,
    date: DateTime<Utc>,
) -> Result<Vec<(EventId, RoomId)>, Error> {
    // Using BTreeSet to guarantee uniqeness
    let mut to_be_deleted =
        BTreeSet::from_iter(Event::get_all_that_ended_before_including_rooms(conn, date).await?);
    to_be_deleted.append(&mut get_recurring_events_that_ended_before(logger, conn, date).await?);
    Ok(Vec::from_iter(to_be_deleted))
}

async fn get_recurring_events_that_ended_before(
    logger: &dyn Log,
    conn: &mut DbConnection,
    date: DateTime<Utc>,
) -> Result<BTreeSet<(EventId, RoomId)>, Error> {
    Ok(Event::get_all_finite_recurring(conn).await?
        .into_iter()
        .filter_map(
            |event| match event.has_last_occurrence_before(date) {
                Ok(true) => Some((event.id, event.room)),
                Ok(false) => None,
                Err(e) => {
                    warn!(log: logger, "Not considering event {} for deletion, because last occurrence date could not be determined: {e}", event.id);
                    None
                }
            },
        )
        .collect())
}

pub(crate) async fn update_user_accounts(
    logger: &dyn Log,
    conn: &mut DbConnection,
    users: Vec<User>,
    kc_users: Vec<opentalk_keycloak_admin::users::User>,
    user_attribute_name: Option<&str>,
) -> Result<(), Error> {
    info!(log: logger, "Updating user accounts");

    let mut kc_user_data: HashMap<String, bool> = kc_users
        .iter()
        .map(|kc_user| {
            let kc_user_id = kc_user
                .get_keycloak_user_id(user_attribute_name)
                .unwrap_or(kc_user.id.as_str())
                .to_owned();

            (kc_user_id, kc_user.enabled)
        })
        .collect();

    let mut updated_users = 0;
    for user in users {
        let kc_user_enabled = kc_user_data.entry(user.oidc_sub).or_default();
        match (user.disabled_since.is_some(), kc_user_enabled) {
            (true, true) => {
                // Enable users if they are disabled in our system but also exist in keycloak
                info!(log: logger, "Enable user: {:?}", user.id);
                let changeset = UpdateUser {
                    disabled_since: Some(None),
                    ..Default::default()
                };
                changeset.apply(conn, user.id).await?;
                updated_users += 1;
            }
            (false, false) => {
                // Disable user if they exist in our system but not in Keycloak
                // Or if they exist in our system but are not enabled in Keycloak
                info!(log: logger, "Disable user: {:?}", user.id);
                let changeset = UpdateUser {
                    disabled_since: Some(Some(Utc::now())),
                    ..Default::default()
                };
                changeset.apply(conn, user.id).await?;
                updated_users += 1;
            }
            _ => {}
        }
    }

    info!(log: logger, "Updated {} user accounts", updated_users);
    Ok(())
}
