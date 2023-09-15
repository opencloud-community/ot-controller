// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::HashSet, sync::Arc};

use chrono::{DateTime, Utc};
use controller_utils::deletion::{Deleter, EventDeleter, RoomDeleter};
use database::{Db, DbConnection};
use db_storage::events::Event;
use kustos::Authz;
use log::Log;
use opentalk_log::{debug, info, warn};
use settings::Settings;
use signaling_core::{ExchangeHandle, ObjectStorage};
use types::core::{EventId, RoomId};

use crate::Error;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum DeleteSelector {
    AdHocCreatedBefore(DateTime<Utc>),
    ScheduledThatEndedBefore(DateTime<Utc>),
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
    let object_storage = ObjectStorage::new(&settings.minio)
        .await
        .map_err(Error::ObjectStorage)?;

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

/// Identify and delete events according to the duration threshold since the
/// last occurrence
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

    let candidates = retrieve_deletion_candidate_events(conn, delete_selector).await?;
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
            warn!(log: logger, "Failed deletion: {e}");
            deleter_failures += 1;
            continue;
        }

        match Event::get_first_for_room(conn, room_id).await {
            Ok(Some(_)) => {}
            Ok(None) => {
                let _ = orphaned_rooms.insert(room_id);
            }
            Err(e) => {
                warn!(log: logger, "Failed to retrieve events connected to room: {e}");
            }
        }
    }

    info!(log: logger, "Deleted {} events", candidate_count);
    if deleter_failures > 0 {
        warn!(log: logger, "{deleter_failures} events could not be deleted due to errors");
    }

    Ok(orphaned_rooms)
}

#[allow(clippy::too_many_arguments)]
async fn delete_orphaned_rooms(
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
            warn!(log: logger, "Failed deletion: {e}");
            deleter_failures += 1;
        }
    }
    info!(log: logger, "Deleted {} orphaned rooms", room_count);
    if deleter_failures > 0 {
        warn!(log: logger, "{deleter_failures} orphaned rooms could not be deleted due to errors");
    }
    Ok(())
}

async fn retrieve_deletion_candidate_events(
    conn: &mut DbConnection,
    delete_selector: DeleteSelector,
) -> Result<Vec<(EventId, RoomId)>, Error> {
    let events = match delete_selector {
        DeleteSelector::AdHocCreatedBefore(delete_before) => {
            Event::get_all_adhoc_created_before_including_rooms(conn, delete_before).await
        }
        DeleteSelector::ScheduledThatEndedBefore(delete_before) => {
            Event::get_all_that_ended_before_including_rooms(conn, delete_before).await
        }
    }?;

    Ok(events)
}
