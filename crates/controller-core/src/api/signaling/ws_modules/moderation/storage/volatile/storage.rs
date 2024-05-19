// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use opentalk_signaling_core::VolatileStaticMemoryStorage;
use parking_lot::RwLock;

use super::memory::MemoryModerationState;
use crate::api::signaling::moderation::storage::ModerationStorage;

static STATE: OnceLock<Arc<RwLock<MemoryModerationState>>> = OnceLock::new();

fn state() -> &'static Arc<RwLock<MemoryModerationState>> {
    STATE.get_or_init(Default::default)
}

#[async_trait(?Send)]
impl ModerationStorage for VolatileStaticMemoryStorage {}
