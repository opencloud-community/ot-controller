// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::ModuleContext;
use super::{SignalingModule, Timestamp};
use crate::api::signaling::ws::runner::Builder;
use crate::api::signaling::ws::{DestroyContext, ExchangePublish};
use actix_http::ws::{CloseCode, Message};
use anyhow::{Context, Result};
use futures::stream::SelectAll;
use serde_json::Value;
use signaling_core::{AnyStream, Event, InitContext, RedisConnection, SignalingMetrics};
use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
use types::{
    core::ParticipantId,
    signaling::{
        control::{state::ControlState, Participant},
        ModuleData, Role,
    },
};

#[derive(Debug, thiserror::Error)]
#[error("invalid module namespace")]
pub struct NoSuchModuleError(pub ());

#[derive(Default)]
pub(super) struct Modules {
    modules: HashMap<&'static str, Box<dyn ModuleCaller>>,
    module_features: HashMap<&'static str, Vec<&'static str>>,
}

impl Modules {
    pub fn get_module_features(&self) -> HashMap<&'static str, Vec<&'static str>> {
        self.module_features.clone()
    }

    pub async fn add_module<M>(&mut self, module: M)
    where
        M: SignalingModule,
    {
        log::debug!("Registering module {}", M::NAMESPACE);

        self.modules
            .insert(M::NAMESPACE, Box::new(ModuleCallerImpl { module }));
        self.module_features
            .insert(M::NAMESPACE, M::get_provided_features());
    }

    pub async fn on_event_targeted(
        &mut self,
        ctx: DynEventCtx<'_>,
        module: &str,
        dyn_event: DynTargetedEvent,
    ) -> Result<(), NoSuchModuleError> {
        let module = self.modules.get_mut(module).ok_or(NoSuchModuleError(()))?;

        if let Err(e) = module.on_event_targeted(ctx, dyn_event).await {
            log::error!("Failed to handle event {:?}", e);
        }

        Ok(())
    }

    pub async fn on_event_broadcast(
        &mut self,
        ctx: DynEventCtx<'_>,
        mut dyn_event: DynBroadcastEvent<'_>,
    ) {
        for module in self.modules.values_mut() {
            let ctx = DynEventCtx {
                id: ctx.id,
                role: ctx.role,
                ws_messages: ctx.ws_messages,
                exchange_publish: ctx.exchange_publish,
                redis_conn: ctx.redis_conn,
                events: ctx.events,
                invalidate_data: ctx.invalidate_data,
                timestamp: ctx.timestamp,
                exit: ctx.exit,
                metrics: ctx.metrics.clone(),
            };

            if let Err(e) = module.on_event_broadcast(ctx, &mut dyn_event).await {
                log::error!("Failed to handle event, {:?}", e);
            }
        }
    }

    pub async fn destroy(&mut self, ctx: DestroyContext<'_>) {
        for (namespace, module) in self.modules.drain() {
            log::debug!("Destroying module {}", namespace);

            module
                .destroy(DestroyContext {
                    redis_conn: ctx.redis_conn,
                    destroy_room: ctx.destroy_room,
                })
                .await;
        }
    }
}

/// Events that are specific to a module
#[derive(Debug)]
pub enum DynTargetedEvent {
    WsMessage(Value),
    ExchangeMessage(Value),
    Ext(Box<dyn Any + 'static>),
}

/// Events that can dispatched to all modules
#[derive(Debug)]
pub enum DynBroadcastEvent<'evt> {
    Joined(
        &'evt ControlState,
        &'evt mut ModuleData,
        &'evt mut Vec<Participant>,
    ),
    Leaving,
    RaiseHand,
    LowerHand,
    ParticipantJoined(&'evt mut Participant),
    ParticipantLeft(ParticipantId),
    ParticipantUpdated(&'evt mut Participant),
    RoleUpdated(Role),
}

/// Untyped version of a ModuleContext which is used in `on_event`
pub(super) struct DynEventCtx<'ctx> {
    pub id: ParticipantId,
    pub role: Role,
    pub timestamp: Timestamp,
    pub ws_messages: &'ctx mut Vec<Message>,
    pub exchange_publish: &'ctx mut Vec<ExchangePublish>,
    pub redis_conn: &'ctx mut RedisConnection,
    pub events: &'ctx mut SelectAll<AnyStream>,
    pub invalidate_data: &'ctx mut bool,
    pub exit: &'ctx mut Option<CloseCode>,
    pub metrics: Arc<SignalingMetrics>,
}

#[async_trait::async_trait(?Send)]
trait ModuleCaller {
    async fn on_event_targeted(
        &mut self,
        ctx: DynEventCtx<'_>,
        dyn_event: DynTargetedEvent,
    ) -> Result<()>;
    async fn on_event_broadcast(
        &mut self,
        ctx: DynEventCtx<'_>,
        dyn_event: &mut DynBroadcastEvent<'_>,
    ) -> Result<()>;
    async fn destroy(self: Box<Self>, ctx: DestroyContext<'_>);
}

struct ModuleCallerImpl<M> {
    pub module: M,
}

impl<M> ModuleCallerImpl<M>
where
    M: SignalingModule,
{
    async fn handle_dyn_targeted_event(
        &mut self,
        ctx: ModuleContext<'_, M>,
        dyn_event: DynTargetedEvent,
    ) -> Result<()> {
        let ctx = ModuleContext {
            role: ctx.role,
            ws_messages: ctx.ws_messages,
            exchange_publish: ctx.exchange_publish,
            redis_conn: ctx.redis_conn,
            events: ctx.events,
            invalidate_data: ctx.invalidate_data,
            exit: ctx.exit,
            timestamp: ctx.timestamp,
            metrics: ctx.metrics,
            m: PhantomData::<fn() -> M>,
        };

        match dyn_event {
            DynTargetedEvent::WsMessage(msg) => {
                let msg = serde_json::from_value(msg).context("Failed to parse WS message")?;
                self.module.on_event(ctx, Event::WsMessage(msg)).await?;
            }
            DynTargetedEvent::ExchangeMessage(msg) => {
                let msg =
                    serde_json::from_value(msg).context("Failed to parse exchange message")?;
                self.module.on_event(ctx, Event::Exchange(msg)).await?;
            }
            DynTargetedEvent::Ext(ext) => {
                self.module
                    .on_event(ctx, Event::Ext(*ext.downcast().expect("invalid ext type")))
                    .await?;
            }
        }

        Ok(())
    }

    async fn handle_dyn_broadcast_event(
        &mut self,
        ctx: ModuleContext<'_, M>,
        dyn_event: &mut DynBroadcastEvent<'_>,
    ) -> Result<()> {
        let ctx = ModuleContext {
            role: ctx.role,
            ws_messages: ctx.ws_messages,
            exchange_publish: ctx.exchange_publish,
            redis_conn: ctx.redis_conn,
            events: ctx.events,
            invalidate_data: ctx.invalidate_data,
            exit: ctx.exit,
            timestamp: ctx.timestamp,
            metrics: ctx.metrics,
            m: PhantomData::<fn() -> M>,
        };

        match dyn_event {
            DynBroadcastEvent::Joined(control_data, module_data, participants) => {
                let mut frontend_data = None;
                let mut participants_data = participants.iter().map(|p| (p.id, None)).collect();

                self.module
                    .on_event(
                        ctx,
                        Event::Joined {
                            control_data,
                            frontend_data: &mut frontend_data,
                            participants: &mut participants_data,
                        },
                    )
                    .await?;

                if let Some(frontend_data) = frontend_data {
                    module_data
                        .insert(&frontend_data)
                        .context("Failed to convert frontend-data to value")?;
                }

                for participant in participants.iter_mut() {
                    if let Some(data) = participants_data.remove(&participant.id).flatten() {
                        participant
                            .module_data
                            .insert(&data)
                            .context("Failed to convert module peer frontend data to value")?;
                    }
                }
            }
            DynBroadcastEvent::Leaving => {
                self.module.on_event(ctx, Event::Leaving).await?;
            }
            DynBroadcastEvent::RaiseHand => {
                self.module.on_event(ctx, Event::RaiseHand).await?;
            }
            DynBroadcastEvent::LowerHand => {
                self.module.on_event(ctx, Event::LowerHand).await?;
            }
            DynBroadcastEvent::ParticipantJoined(participant) => {
                let mut data = None;

                self.module
                    .on_event(ctx, Event::ParticipantJoined(participant.id, &mut data))
                    .await?;

                if let Some(data) = data {
                    participant
                        .module_data
                        .insert(&data)
                        .context("Failed to convert module peer frontend data to value")?;
                }
            }
            DynBroadcastEvent::ParticipantLeft(participant) => {
                self.module
                    .on_event(ctx, Event::ParticipantLeft(*participant))
                    .await?;
            }
            DynBroadcastEvent::ParticipantUpdated(participant) => {
                let mut data = None;

                self.module
                    .on_event(ctx, Event::ParticipantUpdated(participant.id, &mut data))
                    .await?;

                if let Some(data) = data {
                    participant
                        .module_data
                        .insert(&data)
                        .context("Failed to convert module peer frontend data to value")?;
                }
            }
            DynBroadcastEvent::RoleUpdated(role) => {
                self.module.on_event(ctx, Event::RoleUpdated(*role)).await?;
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait(?Send)]
impl<M> ModuleCaller for ModuleCallerImpl<M>
where
    M: SignalingModule,
{
    #[tracing::instrument(skip(self, dyn_ctx, dyn_event), fields(module = %M::NAMESPACE))]
    async fn on_event_targeted(
        &mut self,
        dyn_ctx: DynEventCtx<'_>,
        dyn_event: DynTargetedEvent,
    ) -> Result<()> {
        let mut ws_messages = vec![];

        let ctx = ModuleContext {
            role: dyn_ctx.role,
            timestamp: dyn_ctx.timestamp,
            ws_messages: &mut ws_messages,
            exchange_publish: dyn_ctx.exchange_publish,
            redis_conn: dyn_ctx.redis_conn,
            events: dyn_ctx.events,
            invalidate_data: dyn_ctx.invalidate_data,
            exit: dyn_ctx.exit,
            metrics: Some(dyn_ctx.metrics.clone()),
            m: PhantomData::<fn() -> M>,
        };

        let result = self.handle_dyn_targeted_event(ctx, dyn_event).await;

        let mut ws_messages_serialized = ws_messages
            .into_iter()
            .map(|message| {
                Message::Text(
                    serde_json::to_string(&message)
                        .expect("Failed to convert namespaced to json")
                        .into(),
                )
            })
            .collect();

        dyn_ctx.ws_messages.append(&mut ws_messages_serialized);

        result
    }

    #[tracing::instrument(skip(self, dyn_ctx, dyn_event), fields(module = %M::NAMESPACE))]
    async fn on_event_broadcast(
        &mut self,
        dyn_ctx: DynEventCtx<'_>,
        dyn_event: &mut DynBroadcastEvent<'_>,
    ) -> Result<()> {
        let mut ws_messages = vec![];

        let ctx = ModuleContext {
            role: dyn_ctx.role,
            timestamp: dyn_ctx.timestamp,
            ws_messages: &mut ws_messages,
            exchange_publish: dyn_ctx.exchange_publish,
            redis_conn: dyn_ctx.redis_conn,
            events: dyn_ctx.events,
            invalidate_data: dyn_ctx.invalidate_data,
            exit: dyn_ctx.exit,
            metrics: Some(dyn_ctx.metrics.clone()),
            m: PhantomData::<fn() -> M>,
        };

        let result = self.handle_dyn_broadcast_event(ctx, dyn_event).await;

        let mut ws_messages_serialized = ws_messages
            .into_iter()
            .map(|message| {
                Message::Text(
                    serde_json::to_string(&message)
                        .expect("Failed to convert namespaced to json")
                        .into(),
                )
            })
            .collect();

        dyn_ctx.ws_messages.append(&mut ws_messages_serialized);

        result
    }

    #[tracing::instrument(name = "module_destroy", skip(self, ctx), fields(module = %M::NAMESPACE))]
    async fn destroy(self: Box<Self>, ctx: DestroyContext<'_>) {
        self.module.on_destroy(ctx).await
    }
}

#[async_trait::async_trait(?Send)]
pub trait ModuleBuilder: Send + Sync {
    async fn build(&self, builder: &mut Builder) -> Result<()>;

    fn clone_boxed(&self) -> Box<dyn ModuleBuilder>;

    fn namespace(&self) -> &'static str;

    fn provided_features(&self) -> Vec<&'static str>;
}

pub struct ModuleBuilderImpl<M>
where
    M: SignalingModule,
{
    pub m: PhantomData<fn() -> M>,
    pub params: M::Params,
}

#[async_trait::async_trait(?Send)]
impl<M> ModuleBuilder for ModuleBuilderImpl<M>
where
    M: SignalingModule,
{
    async fn build(&self, builder: &mut Builder) -> Result<()> {
        let ctx = InitContext {
            id: builder.id,
            room: &builder.room,
            breakout_room: builder.breakout_room,
            participant: &builder.participant,
            role: builder.role,
            db: &builder.db,
            storage: &builder.storage,
            authz: &builder.authz,
            exchange_bindings: &mut builder.exchange_bindings,
            events: &mut builder.events,
            redis_conn: &mut builder.redis_conn,
            m: PhantomData::<fn() -> M>,
        };

        if let Some(module) = M::init(ctx, &self.params, builder.protocol).await? {
            builder.modules.add_module(module).await;
        }

        Ok(())
    }

    fn clone_boxed(&self) -> Box<dyn ModuleBuilder> {
        Box::new(Self {
            m: self.m,
            params: self.params.clone(),
        })
    }

    fn namespace(&self) -> &'static str {
        M::NAMESPACE
    }

    fn provided_features(&self) -> Vec<&'static str> {
        M::get_provided_features()
    }
}

impl Clone for Box<dyn ModuleBuilder> {
    fn clone(&self) -> Self {
        (**self).clone_boxed()
    }
}
