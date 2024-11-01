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

use async_trait::async_trait;
use opentalk_controller_service_facade::OpenTalkControllerServiceBackend;

/// The default [`OpenTalkControllerServiceBackend`] implementation.
#[derive(Debug, Default)]
pub struct ControllerBackend;

impl ControllerBackend {
    /// Create a new [`ControllerBackend`].
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl OpenTalkControllerServiceBackend for ControllerBackend {}
