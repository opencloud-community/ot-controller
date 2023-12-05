// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 events endpoints.

mod event_and_instance_id;
mod event_status;
mod instance_id;

pub use event_and_instance_id::EventAndInstanceId;
pub use event_status::EventStatus;
pub use instance_id::InstanceId;

const UTC_DT_FORMAT: &str = "%Y%m%dT%H%M%SZ";
