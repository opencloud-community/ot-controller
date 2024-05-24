// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;

mod any_stream;
mod destroy_context;
mod event;
mod exchange_task;
mod expiring_data;
mod expiring_data_hash_map;
mod init_context;
mod metrics;
mod module_context;
mod object_storage;
mod participant;
mod redis_wrapper;
mod room_lock;
mod runner_id;
mod signaling_module;
mod signaling_room_id;
mod volatile_storage;

#[cfg(feature = "module_tester")]
pub mod module_tester;

pub mod assets;
pub mod control;

pub use any_stream::{any_stream, AnyStream};
pub use destroy_context::DestroyContext;
pub use event::Event;
pub use exchange_task::{Error as ExchangeError, ExchangeHandle, ExchangeTask, SubscriberHandle};
pub use expiring_data::ExpiringData;
pub use expiring_data_hash_map::ExpiringDataHashMap;
pub use init_context::{ExchangeBinding, InitContext};
pub use metrics::SignalingMetrics;
pub use module_context::{ExchangePublish, ModuleContext};
pub use object_storage::{ObjectStorage, ObjectStorageError};
pub use participant::Participant;
pub use redis_wrapper::{RedisConnection, RedisMetrics};
pub use room_lock::{LockError, Locking};
pub use runner_id::RunnerId;
pub use signaling_module::*;
pub use signaling_room_id::SignalingRoomId;
pub use volatile_storage::{VolatileStaticMemoryStorage, VolatileStorage, VolatileStorageBackend};

#[async_trait(?Send)]
pub trait RegisterModules {
    async fn register<E>(registrar: &mut impl ModulesRegistrar<Error = E>) -> Result<(), E>;
}

#[async_trait(?Send)]
pub trait ModulesRegistrar {
    type Error: Send + Sync;

    async fn register<M: SignalingModule>(&mut self) -> Result<(), Self::Error>;
}
