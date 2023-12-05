// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 events endpoints.

mod instance_id;

pub use instance_id::InstanceId;

const UTC_DT_FORMAT: &str = "%Y%m%dT%H%M%SZ";
