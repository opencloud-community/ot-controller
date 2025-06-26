// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage::{
    rooms::Room,
    sip_configs::{NewSipConfig, SipConfig, UpdateSipConfig},
};
use opentalk_types_common::{call_in::CallInId, rooms::RoomId};

use crate::Result;

/// A trait for retrieving and storing room sip config entities.
#[async_trait::async_trait]
pub trait RoomSipConfigInventory {
    /// Get the SIP config for a room.
    async fn get_room_sip_config(&mut self, room_id: RoomId) -> Result<Option<SipConfig>>;

    /// Get the SIP config by the call-in id.
    async fn get_room_sip_config_with_room(
        &mut self,
        call_in_id: CallInId,
    ) -> Result<Option<(SipConfig, Room)>>;

    /// Create a SIP config for a room
    async fn create_room_sip_config(&mut self, sip_config: NewSipConfig) -> Result<SipConfig>;

    /// Update a room SIP config.
    async fn update_room_sip_config(
        &mut self,
        room_id: RoomId,
        sip_config: UpdateSipConfig,
    ) -> Result<Option<SipConfig>>;

    /// Delete a room SIP config.
    async fn delete_room_sip_config(&mut self, room_id: RoomId) -> Result<()>;
}
