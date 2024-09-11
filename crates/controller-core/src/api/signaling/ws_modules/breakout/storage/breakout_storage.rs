// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use async_trait::async_trait;
use opentalk_signaling_core::{
    control::storage::{ControlStorageParticipantAttributesRaw, ControlStorageParticipantSet},
    SignalingModuleError,
};
use opentalk_types_common::rooms::RoomId;

use super::BreakoutConfig;

#[async_trait(?Send)]
pub trait BreakoutStorage:
    ControlStorageParticipantSet + ControlStorageParticipantAttributesRaw
{
    /// Set the breakout configuration.
    /// If this is an expiring breakout session, the "real" breakout expiry will be returned.
    /// Some implementations (such as redis) cannot cope with expiry below seconds resolution,
    /// and don't accept 0 seconds. Therefore, the returned value is roughly what the storage
    /// backend will use as the real expiry time. Because at the moment when this function
    /// returns, the expiry duration already started, it should be sufficient to wait *exactly*
    /// that time, then the value should no longer be available.
    async fn set_breakout_config(
        &mut self,
        room: RoomId,
        config: &BreakoutConfig,
    ) -> Result<Option<Duration>, SignalingModuleError>;

    async fn get_breakout_config(
        &mut self,
        room: RoomId,
    ) -> Result<Option<BreakoutConfig>, SignalingModuleError>;

    async fn del_breakout_config(&mut self, room: RoomId) -> Result<bool, SignalingModuleError>;
}
