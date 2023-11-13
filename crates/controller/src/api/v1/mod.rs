// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! REST API v1
//!
//! Current Endpoints. See their respective function:
//! - `/auth/login` ([GET](auth::get_login), [POST](auth::post_login))
//! - `/rooms` ([GET](rooms::accessible), [POST](rooms::new))
//! - `/rooms/{room_id}` ([GET](rooms::get), [PATCH](rooms::patch))
//! - `/rooms/{room_id}/start` ([POST](rooms::start))
//! - `/rooms/{room_id}/start_invited` ([POST](rooms::start_invited))
//! - `/rooms/{room_id}/invites ([GET](invites::get_invites), [POST](invites::add_invite))
//! - `/rooms/{room_id}/invites/{invite_code} ([GET](invites::get_invite), [PUT](invites::update_invite), [DELETE](invites::delete_invite)])
//! - `/rooms/{room_id}/sip ([GET](sip_configs::get), [PUT](sip_configs::put), [DELETE](sip_configs::delete))
//! - `/turn` ([GET](turn::get))
//! - `/users/me`([GET](users::get_me), [PATCH](users::patch_me))
//! - `/users/{user_id}` ([GET](users::get_user))
//! - `/users/find` ([GET](users::find))
//! - `/services/call_in/start ([POST](services::call_in::start))

pub use response::{ApiResponse, DefaultApiResult};

pub mod assets;
pub mod auth;
pub mod events;
pub mod invites;
pub mod middleware;
pub mod response;
pub mod rooms;
pub mod services;
pub mod sip_configs;
pub mod streaming_targets;
pub mod turn;
pub mod users;
mod util;
