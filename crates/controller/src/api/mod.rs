// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Modules for external HTTP APIs
//!
//! Versions REST APIs are in v{VERSION}
//! APIs for use with our own frontend lie in internal
//! These directory map to the path prefix `/internal` or `/v1`

pub(crate) mod internal;
mod util;
#[macro_use]
pub mod signaling;
pub mod v1;
