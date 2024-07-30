// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::control::storage::ControlStorage;

#[async_trait(?Send)]
pub(crate) trait MeetingReportStorage: ControlStorage {}

impl<T: ControlStorage> MeetingReportStorage for T {}
