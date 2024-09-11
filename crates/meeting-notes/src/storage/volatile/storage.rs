// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId, VolatileStaticMemoryStorage};
use opentalk_types_signaling::ParticipantId;
use parking_lot::RwLock;

use super::memory::MemoryMeetingNotesState;
use crate::{
    storage::{meeting_notes_storage::MeetingNotesStorage, InitState},
    SessionInfo,
};

static STATE: OnceLock<Arc<RwLock<MemoryMeetingNotesState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemoryMeetingNotesState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl MeetingNotesStorage for VolatileStaticMemoryStorage {
    #[tracing::instrument(name = "set_meeting_notes_group", skip(self))]
    async fn group_set(
        &mut self,
        room: SignalingRoomId,
        group: &str,
    ) -> Result<(), SignalingModuleError> {
        state().write().group_set(room, group);
        Ok(())
    }

    #[tracing::instrument(name = "get_meeting_notes_group", skip(self))]
    async fn group_get(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<String>, SignalingModuleError> {
        Ok(state().read().group_get(room))
    }

    #[tracing::instrument(name = "delete_meeting_notes_group", skip(self))]
    async fn group_delete(&mut self, room: SignalingRoomId) -> Result<(), SignalingModuleError> {
        state().write().group_delete(room);
        Ok(())
    }

    #[tracing::instrument(name = "meeting_notes_try_start_init", skip(self))]
    async fn try_start_init(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<InitState>, SignalingModuleError> {
        Ok(state().write().init_get_or_default(room))
    }

    #[tracing::instrument(name = "meeting_notes_set_initialized", skip(self))]
    async fn set_initialized(&mut self, room: SignalingRoomId) -> Result<(), SignalingModuleError> {
        state().write().set_initialized(room);
        Ok(())
    }

    #[tracing::instrument(name = "get_meeting_notes_init_state", skip(self))]
    async fn init_get(
        &mut self,
        room: SignalingRoomId,
    ) -> Result<Option<InitState>, SignalingModuleError> {
        Ok(state().read().init_get(room))
    }

    #[tracing::instrument(name = "delete_meeting_notes_init_state", skip(self))]
    async fn init_delete(&mut self, room: SignalingRoomId) -> Result<(), SignalingModuleError> {
        state().write().init_delete(room);
        Ok(())
    }

    #[tracing::instrument(name = "get_meeting_notes_session_info", skip(self))]
    async fn session_get(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<Option<SessionInfo>, SignalingModuleError> {
        Ok(state().read().session_get(room, participant))
    }

    #[tracing::instrument(name = "set_meeting_notes_session_info", skip(self))]
    async fn session_set(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
        session_info: &SessionInfo,
    ) -> Result<(), SignalingModuleError> {
        state()
            .write()
            .session_set(room, participant, session_info.clone());
        Ok(())
    }

    #[tracing::instrument(name = "get_del_meeting_notes_session_info", skip(self))]
    async fn session_delete(
        &mut self,
        room: SignalingRoomId,
        participant: ParticipantId,
    ) -> Result<Option<SessionInfo>, SignalingModuleError> {
        Ok(state().write().session_delete(room, participant))
    }
}

#[cfg(test)]
mod test {
    use opentalk_signaling_core::VolatileStaticMemoryStorage;
    use serial_test::serial;

    use super::{super::super::test_common, state};

    fn storage() -> VolatileStaticMemoryStorage {
        state().write().reset();
        VolatileStaticMemoryStorage
    }

    #[tokio::test]
    #[serial]
    async fn group() {
        test_common::group(&mut storage()).await;
    }

    #[tokio::test]
    #[serial]
    async fn init() {
        test_common::init(&mut storage()).await;
    }

    #[tokio::test]
    #[serial]
    async fn session() {
        test_common::session(&mut storage()).await;
    }
}
