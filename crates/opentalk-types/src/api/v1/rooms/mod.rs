// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 rooms endpoints.

mod post_rooms_start_invited_request;
mod post_rooms_start_invited_request_body;
mod post_rooms_start_request;
mod post_rooms_start_request_body;
mod rooms_start_response;
mod start_room_error;

pub mod sip_config_resource;
pub mod streaming_targets;

pub use post_rooms_start_invited_request::PostRoomsStartInvitedRequest;
pub use post_rooms_start_invited_request_body::PostRoomsStartInvitedRequestBody;
pub use post_rooms_start_request::PostRoomsStartRequest;
pub use post_rooms_start_request_body::PostRoomsStartRequestBody;
pub use rooms_start_response::RoomsStartResponse;
pub use start_room_error::StartRoomError;
