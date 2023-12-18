// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used in different areas of the OpenTalk
//! API V1 endpoints.

pub mod assets;
pub mod auth;
pub mod events;
pub mod invites;
pub mod pagination;
pub mod rooms;
pub mod services;
pub mod streaming_targets;
pub mod turn;
pub mod users;
pub mod utils;

mod cursor;

pub use cursor::Cursor;
