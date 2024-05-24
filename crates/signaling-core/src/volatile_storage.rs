// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use either::Either;

use crate::RedisConnection;

#[derive(Clone)]
pub struct VolatileStaticMemoryStorage;

pub type VolatileStorage = Either<VolatileStaticMemoryStorage, RedisConnection>;
