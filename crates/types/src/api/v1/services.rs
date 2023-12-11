// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 events endpoints.

mod call_in_start_request_body;
mod call_in_start_response;
mod recorder_start_body;

pub use call_in_start_request_body::CallInStartRequestBody;
pub use call_in_start_response::CallInStartResponse;
pub use recorder_start_body::RecorderStartBody;
