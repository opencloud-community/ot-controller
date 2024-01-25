// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! OpenTalk controller REST API v1 service handlers

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
