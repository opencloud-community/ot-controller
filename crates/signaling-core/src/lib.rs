// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use async_trait::async_trait;

mod any_stream;
mod destroy_context;
mod event;
mod exchange_task;
mod init_context;
mod metrics;
mod module_context;
mod object_storage;
mod participant;
mod redis_wrapper;
mod signaling_module;
mod signaling_room_id;

#[cfg(feature = "module_tester")]
pub mod module_tester;

pub mod assets;
pub mod control;

pub use any_stream::{any_stream, AnyStream};
pub use destroy_context::DestroyContext;
pub use event::Event;
pub use exchange_task::{ExchangeHandle, ExchangeTask, SubscriberHandle};
pub use init_context::{ExchangeBinding, InitContext};
pub use metrics::SignalingMetrics;
pub use module_context::{ExchangePublish, ModuleContext};
pub use object_storage::ObjectStorage;
pub use participant::Participant;
pub use redis_wrapper::{RedisConnection, RedisMetrics};
pub use signaling_module::{SignalingModule, SignalingModuleInitData};
pub use signaling_room_id::SignalingRoomId;

#[async_trait(?Send)]
pub trait RegisterModules {
    async fn register(registrar: &mut impl ModulesRegistrar) -> Result<()>;
}

#[async_trait(?Send)]
pub trait ModulesRegistrar {
    async fn register<M: SignalingModule>(&mut self) -> Result<()>;
}
