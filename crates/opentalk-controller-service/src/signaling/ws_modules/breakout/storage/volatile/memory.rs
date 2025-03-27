// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::HashMap, time::Duration};

use opentalk_signaling_core::ExpiringData;
use opentalk_types_common::rooms::RoomId;

use crate::signaling::ws_modules::breakout::storage::BreakoutConfig;

#[derive(Debug, Clone, Default)]
pub(super) struct MemoryBreakoutState {
    configs: HashMap<RoomId, ExpiringData<BreakoutConfig>>,
}

impl MemoryBreakoutState {
    #[cfg(test)]
    pub fn reset(&mut self) {
        *self = Default::default();
    }

    pub(super) fn set_config(&mut self, room: RoomId, config: &BreakoutConfig) -> Option<Duration> {
        if let Some(duration) = config.duration {
            _ = self
                .configs
                .insert(room, ExpiringData::new_ex(config.clone(), duration));
            Some(duration)
        } else {
            _ = self.configs.insert(room, ExpiringData::new(config.clone()));
            None
        }
    }

    pub(super) fn get_config(&self, room: RoomId) -> Option<BreakoutConfig> {
        self.configs
            .get(&room)
            .and_then(ExpiringData::value)
            .cloned()
    }

    pub(super) fn del_config(&mut self, room: RoomId) -> bool {
        self.configs.remove(&room).is_some()
    }
}
