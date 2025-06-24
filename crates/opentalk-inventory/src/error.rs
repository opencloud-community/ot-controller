// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains the error type that is returned form the traits defined in this crate.
//!
//! The module is public so that implementors of the traits can create variants of it
//! using the types defined by [snafu], e.g. [StorageBackendSnafu] or [JsonOperationSnafu].
//!
//! [snafu::whatever] can be used with this type as well if necessary.

use opentalk_database::DatabaseError;
use opentalk_db_storage::module_resources::JsonOperationError;
use snafu::Snafu;

/// The error returned from function calls to the storage facade provider.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    /// An error happened in the storage backend.
    StorageBackend {
        /// The cause of the error
        // TODO: this needs to be replaced by a `Box<dyn std::error::Error + Sync + Send>`.
        // TODO: I wanted to insert that right away, but didn't get the compiler errors fixed,
        // TODO: so this is a task for later.
        source: DatabaseError,
    },
    /// An error happened when applying JSON operations to a module resource.
    JsonOperation {
        /// The cause of the error
        // TODO: this needs to be replaced by a `Box<dyn std::error::Error + Sync + Send>`.
        // TODO: I wanted to insert that right away, but didn't get the compiler errors fixed,
        // TODO: so this is a task for later.
        source: JsonOperationError,
    },

    /// An error occurred when attempting to begin, rollback or finish a transaction.
    BrokenTransactionManager,

    /// A custom error with just a message.
    #[snafu(whatever, display("{message}"))]
    Custom {
        /// The custom error message.
        message: String,

        /// An optional source error.
        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>,Some)))]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}
