// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 events endpoints.

mod call_in_start_request_body;
mod recorder_start_body;
mod service_start_response;

pub use call_in_start_request_body::CallInStartRequestBody;
pub use recorder_start_body::RecorderStartBody;
pub use service_start_response::ServiceStartResponse;
