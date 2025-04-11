// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Modules for external HTTP APIs
//!
//! Versions REST APIs are in v{VERSION}
//! APIs for use with our own frontend lie in internal
//! These directory map to the path prefix `/internal` or `/v1`

mod util;
#[macro_use]
pub mod signaling;
pub mod headers;
pub mod responses;
pub mod upload;
pub mod v1;
pub mod well_known;
