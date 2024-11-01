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

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

/// Trait implemented by OpenTalk controller service backends
#[async_trait]
pub trait OpenTalkControllerServiceBackend: Send + Sync {}

/// Thread-safe handle to a [`OpenTalkControllerServiceBackend`] implementation.
#[derive(Clone)]
pub struct OpenTalkControllerService {
    _backend: Arc<RwLock<dyn OpenTalkControllerServiceBackend>>,
}

impl std::fmt::Debug for OpenTalkControllerService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OpenTalkControllerService")
    }
}

impl OpenTalkControllerService {
    /// Create a new [`OpenTalkControllerService`] holding a type that implements [`OpenTalkControllerServiceBackend`].
    pub fn new<B: OpenTalkControllerServiceBackend + 'static>(backend: B) -> Self {
        Self {
            _backend: Arc::new(RwLock::new(backend)),
        }
    }
}
