// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 events endpoints.

mod call_in;
mod recording;
mod service_start_response;
mod upload_render_query;

pub use call_in::StartRequestBody;
pub use recording::StartBody;
pub use service_start_response::ServiceStartResponse;
pub use upload_render_query::UploadRenderQuery;
