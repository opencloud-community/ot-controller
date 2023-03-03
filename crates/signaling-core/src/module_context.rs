// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{marker::PhantomData, sync::Arc};

use actix_http::ws::CloseCode;
use futures::{stream::SelectAll, Stream};
use serde::Serialize;
use types::{
    core::Timestamp,
    signaling::{NamespacedEvent, Role},
};

use crate::{any_stream, AnyStream, RedisConnection, SignalingMetrics, SignalingModule};

#[derive(Debug, Clone)]
pub struct ExchangePublish {
    pub routing_key: String,
    pub message: String,
}

/// Context passed to the module
///
/// Can be used to send websocket messages
pub struct ModuleContext<'ctx, M>
where
    M: SignalingModule,
{
    pub role: Role,
    pub ws_messages: &'ctx mut Vec<NamespacedEvent<'static, M::Outgoing>>,
    pub timestamp: Timestamp,
    pub exchange_publish: &'ctx mut Vec<ExchangePublish>,
    pub redis_conn: &'ctx mut RedisConnection,
    pub events: &'ctx mut SelectAll<AnyStream>,
    pub invalidate_data: &'ctx mut bool,
    pub exit: &'ctx mut Option<CloseCode>,
    pub metrics: Option<Arc<SignalingMetrics>>,
    pub m: PhantomData<fn() -> M>,
}

impl<M> ModuleContext<'_, M>
where
    M: SignalingModule,
{
    pub fn role(&self) -> Role {
        self.role
    }

    /// Queue a outgoing message to be sent via the websocket
    /// after exiting the `on_event` function
    pub fn ws_send(&mut self, message: impl Into<M::Outgoing>) {
        self.ws_send_overwrite_timestamp(message, self.timestamp)
    }

    /// Similar to `ws_send` but sets an explicit timestamp
    pub fn ws_send_overwrite_timestamp(
        &mut self,
        message: impl Into<M::Outgoing>,
        timestamp: Timestamp,
    ) {
        self.ws_messages.push(NamespacedEvent {
            namespace: M::NAMESPACE,
            timestamp,
            payload: message.into(),
        });
    }

    /// Queue a outgoing message to be sent via the message exchange
    pub fn exchange_publish(&mut self, routing_key: String, message: M::ExchangeMessage) {
        self.exchange_publish_any(
            routing_key,
            NamespacedEvent {
                namespace: M::NAMESPACE,
                timestamp: self.timestamp,
                payload: message,
            },
        );
    }

    /// Queue any serializable outgoing message to be sent via the message exchange
    pub fn exchange_publish_any(&mut self, routing_key: String, message: impl Serialize) {
        self.exchange_publish.push(ExchangePublish {
            routing_key,
            message: serde_json::to_string(&message).expect("value must be serializable to json"),
        });
    }

    /// Access to the storage of the room
    pub fn redis_conn(&mut self) -> &mut RedisConnection {
        self.redis_conn
    }

    /// Add a custom event stream which return `M::ExtEvent`
    pub fn add_event_stream<S>(&mut self, stream: S)
    where
        S: Stream<Item = M::ExtEvent> + 'static,
    {
        self.events.push(any_stream(M::NAMESPACE, stream));
    }

    /// Signals that the data related to the participant has changed
    pub fn invalidate_data(&mut self) {
        *self.invalidate_data = true;
    }

    pub fn exit(&mut self, code: Option<CloseCode>) {
        *self.exit = Some(code.unwrap_or(CloseCode::Normal));
    }

    pub fn metrics(&self) -> Option<&Arc<SignalingMetrics>> {
        self.metrics.as_ref()
    }

    /// Returns the Timestamp of the event which triggered the `on_event` handler.
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
}
