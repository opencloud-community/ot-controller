// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! OpenTalk Controller service
//!
//! This crate contains the default OpenTalk Controller backend implementation.

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

pub mod controller_backend;
pub mod helpers;

pub use controller_backend::ControllerBackend;
pub use helpers::{email_to_libravatar_url, require_feature, ToUserProfile};
use snafu::{Backtrace, Snafu};

/// Send and Sync variant of [`snafu::Whatever`]
#[derive(Debug, Snafu)]
#[snafu(whatever)]
#[snafu(display("{message}"))]
#[snafu(provide(opt, ref, chain, dyn std::error::Error => source.as_deref()))]
pub struct Whatever {
    #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, Some)))]
    #[snafu(provide(false))]
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
    message: String,
    backtrace: Backtrace,
}
