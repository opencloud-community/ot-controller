// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::VolatileStaticMemoryStorage;

use crate::api::signaling::recording::storage::RecordingStorage;

#[async_trait(?Send)]
impl RecordingStorage for VolatileStaticMemoryStorage {}
