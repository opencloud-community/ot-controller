// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use either::Either;

use crate::RedisConnection;

pub trait VolatileStorage {}

#[derive(Clone)]
pub struct VolatileStaticMemoryStorage;

impl VolatileStorage for VolatileStaticMemoryStorage {}

pub type VolatileStorageBackend = Either<VolatileStaticMemoryStorage, RedisConnection>;

struct VolatileBackendStorage {
    backend: Either<VolatileStaticMemoryStorage, RedisConnection>,
}

impl VolatileBackendStorage {}
