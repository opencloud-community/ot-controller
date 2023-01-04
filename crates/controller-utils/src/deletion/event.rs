// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Functionality to delete events including all associated resources

use database::{DatabaseError, DbConnection};
use db_storage::events::{shared_folders::EventSharedFolder, Event};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection};
use kustos::{Authz, Resource as _, ResourceId};
use kustos_shared::access::AccessMethod;
use log::Log;
use opentalk_log::{debug, warn};
use settings::Settings;
use signaling_core::{control, ExchangeHandle, ObjectStorage};
use types::core::{EventId, UserId};

use super::{shared_folders::delete_shared_folders, Deleter, Error};

/// Delete an event by id including the corresponding room and resources it
/// references.
#[derive(Debug)]
pub struct EventDeleter {
    event_id: EventId,
    fail_on_shared_folder_deletion_error: bool,
}

impl EventDeleter {
    /// Create a new `EventDeleter`
    ///
    /// If `fail_on_shared_folder_deletion` is true, the deletion will fail as soon
    /// as a shared folder can not be deleted from the external storage system.
    ///
    /// Otherwise just warnings will be logged, but the deletion is considered successful.
    pub fn new(event_id: EventId, fail_on_shared_folder_deletion_error: bool) -> Self {
        Self {
            event_id,
            fail_on_shared_folder_deletion_error,
        }
    }
}

/// A struct holding the information that was collected during the database
/// commit preparation.
#[derive(Debug)]
pub struct EventDeleterPreparedCommit {
    resources: Vec<ResourceId>,
    linked_shared_folder: Option<EventSharedFolder>,
}

impl EventDeleterPreparedCommit {
    async fn detect_race_condition(
        &self,
        conn: &mut DbConnection,
        event_id: EventId,
    ) -> Result<(), DatabaseError> {
        const ERROR_MESSAGE: &str = "Race-condition during database commit preparation";

        let current_shared_folder = EventSharedFolder::get_for_event(conn, event_id).await?;
        if current_shared_folder != self.linked_shared_folder {
            return Err(DatabaseError::custom(ERROR_MESSAGE));
        }

        Ok(())
    }
}

/// A struct holding the information that was collected during database commit.
#[derive(Debug)]
pub struct EventDeleterCommitOutput {
    resources: Vec<ResourceId>,
}

#[async_trait::async_trait]
impl Deleter for EventDeleter {
    type PreparedCommit = EventDeleterPreparedCommit;
    type CommitOutput = EventDeleterCommitOutput;

    async fn prepare_commit(
        &self,
        _logger: &dyn Log,
        conn: &mut DbConnection,
    ) -> Result<Self::PreparedCommit, Error> {
        let linked_shared_folder = EventSharedFolder::get_for_event(conn, self.event_id).await?;

        let resources = associated_resource_ids(self.event_id)
            .into_iter()
            .chain(
                linked_shared_folder
                    .iter()
                    .map(|f| f.event_id.resource_id().with_suffix("/shared_folder")),
            )
            .collect::<Vec<_>>();

        Ok(EventDeleterPreparedCommit {
            resources,
            linked_shared_folder,
        })
    }

    async fn check_permissions(
        &self,
        _prepared_commit: &Self::PreparedCommit,
        _logger: &dyn Log,
        authz: &Authz,
        user_id: Option<UserId>,
    ) -> Result<(), Error> {
        let user_id = match user_id {
            Some(user_id) => user_id,
            None => return Ok(()),
        };

        let event_id = self.event_id;
        let checked = authz
            .check_user(user_id, event_id.resource_id(), AccessMethod::DELETE)
            .await?;

        if !checked {
            return Err(Error::Forbidden);
        }

        Ok(())
    }

    async fn pre_commit(
        &self,
        prepared_commit: &Self::PreparedCommit,
        logger: &dyn Log,
        conn: &mut DbConnection,
        exchange_handle: ExchangeHandle,
        settings: &Settings,
    ) -> Result<(), Error> {
        let event = Event::get(conn, self.event_id).await?;
        let room_id = event.room;

        let message = types::signaling::NamespacedEvent {
            namespace: control::NAMESPACE,
            timestamp: types::core::Timestamp::now(),
            payload: control::exchange::Message::RoomDeleted,
        };

        if let Err(e) = exchange_handle.publish(
            control::exchange::global_room_all_participants(room_id),
            serde_json::to_string(&message).expect("Failed to convert namespaced to json"),
        ) {
            warn!(log: logger, "Failed to publish message to exchange, {}", e);
        }

        delete_shared_folders(
            logger,
            settings,
            prepared_commit
                .linked_shared_folder
                .as_ref()
                .map(core::slice::from_ref)
                .unwrap_or_default(),
            self.fail_on_shared_folder_deletion_error,
        )
        .await?;
        Ok(())
    }

    async fn commit_to_database(
        &self,
        prepared_commit: Self::PreparedCommit,
        logger: &dyn Log,
        conn: &mut DbConnection,
    ) -> Result<Self::CommitOutput, Error> {
        debug!(log: logger, "Deleting all database resources");

        let event_id = self.event_id;

        let transaction_result: Result<Vec<ResourceId>, DatabaseError> = conn
            .transaction(|conn| {
                async move {
                    prepared_commit
                        .detect_race_condition(conn, event_id)
                        .await?;

                    debug!(log: logger, "Deleting shared folders from database");
                    EventSharedFolder::delete_by_event_ids(conn, &[event_id]).await?;
                    debug!(log: logger, "Deleting event from database");
                    Event::delete_by_id(conn, event_id).await?;

                    Ok(prepared_commit.resources)
                }
                .scope_boxed()
            })
            .await;

        let resources = transaction_result?;

        Ok(EventDeleterCommitOutput { resources })
    }

    async fn post_commit(
        &self,
        commit_output: EventDeleterCommitOutput,
        logger: &dyn Log,
        _settings: &Settings,
        authz: &Authz,
        _storage: &ObjectStorage,
    ) -> Result<(), Error> {
        debug!(log: logger, "Deleting auth information");
        let _removed_count = authz
            .remove_explicit_resources(commit_output.resources)
            .await?;

        Ok(())
    }
}

pub fn associated_resource_ids(event_id: EventId) -> impl IntoIterator<Item = ResourceId> {
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
