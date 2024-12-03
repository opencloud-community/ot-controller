// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! OpenTalk Controller service facade
//!
//! This crate contains traits and data types that provide the service facade
//! which is used by the OpenTalk Controller to provide the Web API.

#![deny(
    bad_style,
    missing_debug_implementations,
    missing_docs,
    overflowing_literals,
    patterns_in_fns_without_body,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]

mod controller_service;
mod controller_service_backend;

pub use controller_service::OpenTalkControllerService;
pub use controller_service_backend::OpenTalkControllerServiceBackend;
