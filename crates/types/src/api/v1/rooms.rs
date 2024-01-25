// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 rooms endpoints.

mod get_room_event_request;
mod get_room_event_response;
mod patch_rooms_request_body;
mod post_rooms_request_body;
mod post_rooms_start_invited_request_body;
mod post_rooms_start_request_body;
mod room_resource;
mod rooms_start_response;
mod start_room_error;

pub mod sip_config_resource;
pub mod streaming_targets;

pub use get_room_event_request::GetRoomEventRequest;
pub use get_room_event_response::GetRoomEventResponse;
pub use patch_rooms_request_body::PatchRoomsRequestBody;
pub use post_rooms_request_body::PostRoomsRequestBody;
pub use post_rooms_start_invited_request_body::PostRoomsStartInvitedRequestBody;
pub use post_rooms_start_request_body::PostRoomsStartRequestBody;
pub use room_resource::RoomResource;
pub use rooms_start_response::RoomsStartResponse;
pub use start_room_error::StartRoomError;
