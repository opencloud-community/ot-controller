// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_db_storage::{
    events::{
        Event, EventException, EventExceptionId, EventFavorite, EventInvite,
        EventTrainingParticipationReportParameterSet, GetEventsCursor, NewEvent, NewEventException,
        NewEventFavorite, UpdateEvent, UpdateEventException, shared_folders::EventSharedFolder,
    },
    rooms::Room,
    sip_configs::SipConfig,
    tariffs::Tariff,
    users::User,
};
use opentalk_inventory::{EventInventory, error::StorageBackendSnafu};
use opentalk_types_common::{
    events::{EventId, invites::EventInviteStatus},
    rooms::RoomId,
    time::Timestamp,
    training_participation_report::TrainingParticipationReportParameterSet,
    users::UserId,
};
use snafu::ResultExt as _;

use crate::{DatabaseConnection, Result};

#[async_trait::async_trait]
impl EventInventory for DatabaseConnection {
    #[tracing::instrument(err, skip_all)]
    async fn create_event(&mut self, new_event: NewEvent) -> Result<Event> {
        new_event
            .insert(&mut self.inner)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn update_event(&mut self, event_id: EventId, event: UpdateEvent) -> Result<Event> {
        event
            .apply(&mut self.inner, event_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_event(&mut self, event_id: EventId) -> Result<Event> {
        Event::get(&mut self.inner, event_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_event_for_room(&mut self, room_id: RoomId) -> Result<Option<Event>> {
        Event::get_for_room(&mut self.inner, room_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_event_id_for_room(&mut self, room_id: RoomId) -> Result<Option<EventId>> {
        Event::get_id_for_room(&mut self.inner, room_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_event_with_room_and_sip_config(
        &mut self,
        event_id: EventId,
    ) -> Result<(Event, Room, Option<SipConfig>)> {
        Event::get_with_room(&mut self.inner, event_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_event_with_related_items(
        &mut self,
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
        Event::get_with_related_items(&mut self.inner, user_id, event_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_all_events_updated_by_user(&mut self, user_id: UserId) -> Result<Vec<Event>> {
        Event::get_all_updated_by_user(&mut self.inner, user_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_all_adhoc_event_ids_with_room_ids_created_before(
        &mut self,
        created_before: Timestamp,
    ) -> Result<Vec<(EventId, RoomId)>> {
        Event::get_all_adhoc_created_before_including_rooms(&mut self.inner, created_before.into())
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_all_scheduled_event_ids_with_room_ids_ended_before(
        &mut self,
        ended_before: Timestamp,
    ) -> Result<Vec<(EventId, RoomId)>> {
        Event::get_all_that_ended_before_including_rooms(&mut self.inner, ended_before.into())
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_all_event_ids_with_room_ids_created_by_user(
        &mut self,
        user_id: UserId,
    ) -> Result<Vec<(EventId, RoomId)>> {
        Event::get_all_for_creator_including_rooms(&mut self.inner, user_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_all_finite_recurring_events(&mut self) -> Result<Vec<Event>> {
        Event::get_all_finite_recurring(&mut self.inner)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_all_events_for_user_paginated(
        &mut self,
        user: &User,
        only_favorites: bool,
        invite_status_filter: BTreeSet<EventInviteStatus>,
        time_min: Option<Timestamp>,
        time_max: Option<Timestamp>,
        created_before: Option<Timestamp>,
        created_after: Option<Timestamp>,
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
        Event::get_all_for_user_paginated(
            &mut self.inner,
            user,
            only_favorites,
            Vec::from_iter(invite_status_filter),
            time_min.map(Into::into),
            time_max.map(Into::into),
            created_before.map(Into::into),
            created_after.map(Into::into),
            adhoc,
            time_independent,
            cursor,
            limit,
        )
        .await
        .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn create_event_exception(
        &mut self,
        event_exception: NewEventException,
    ) -> Result<EventException> {
        event_exception
            .insert(&mut self.inner)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_event_exceptions(
        &mut self,
        event_id: EventId,
        timestamps: &[Timestamp],
    ) -> Result<Vec<EventException>> {
        let timestamps: Vec<_> = timestamps.iter().map(|v| *v.as_ref()).collect();
        EventException::get_all_for_event(&mut self.inner, event_id, &timestamps)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_event_exception(
        &mut self,
        event_id: EventId,
        instance_id_timestamp: Timestamp,
    ) -> Result<Option<EventException>> {
        EventException::get_for_event(&mut self.inner, event_id, instance_id_timestamp.into())
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn update_event_exception(
        &mut self,
        event_exception_id: EventExceptionId,
        event_exception: UpdateEventException,
    ) -> Result<EventException> {
        event_exception
            .apply(&mut self.inner, event_exception_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn delete_event_exceptions_for_event(&mut self, event_id: EventId) -> Result<()> {
        EventException::delete_all_for_event(&mut self.inner, event_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn delete_event_for_room(&mut self, room_id: RoomId) -> Result<()> {
        Event::delete_for_room(&mut self.inner, room_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn create_event_favorite_for_user(
        &mut self,
        event_id: EventId,
        user_id: UserId,
    ) -> Result<bool> {
        NewEventFavorite { event_id, user_id }
            .try_insert(&mut self.inner)
            .await
            .context(StorageBackendSnafu)
            .map(|v| v.is_some())
    }

    #[tracing::instrument(err, skip_all)]
    async fn delete_event_favorite_for_user(
        &mut self,
        event_id: EventId,
        user_id: UserId,
    ) -> Result<bool> {
        EventFavorite::delete_by_id(&mut self.inner, user_id, event_id)
            .await
            .context(StorageBackendSnafu)
    }
}
