// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Includes an enforcer that supports a background task which reloads the adapter every n seconds
use std::{collections::HashMap, convert::TryInto, mem::take, sync::Arc, time::Instant};

use async_trait::async_trait;
use casbin::{
    rhai::ImmutableString, Adapter, CoreApi, Effector, EnforceArgs, Enforcer, Event, EventData,
    EventEmitter, Filter, InternalApi, MgmtApi, Model, RbacApi, Result as CasbinResult,
    RoleManager, TryIntoAdapter, TryIntoModel,
};
use futures::StreamExt;
use kustos_shared::internal::{ToCasbin, ToCasbinString};
use lapin::{
    options::{
        BasicConsumeOptions, BasicPublishOptions, ExchangeDeclareOptions, QueueBindOptions,
        QueueDeclareOptions,
    },
    protocol::basic::AMQPProperties,
    types::{FieldTable, ShortString},
    ExchangeKind,
};
use lapin_pool::{RabbitMqChannel, RabbitMqPool};
use parking_lot as pl;
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{mpsc, RwLock},
    task::JoinHandle,
};
use uuid::Uuid;

use super::rbac_api_ex::RbacApiEx;
use crate::{metrics::KustosMetrics, PolicyUser, UserPolicy};

const EXCHANGE_NAME: &str = "opentalk-acl-updates";

type EventCallback = Box<dyn FnMut(&mut SyncedEnforcer, EventData) + Send + Sync>;

#[derive(Clone, Serialize, Deserialize)]
pub enum KustosEventData {
    AddPolicy(String, String, Vec<String>),
    AddPolicies(String, String, Vec<Vec<String>>),
    RemovePolicy(String, String, Vec<String>),
    RemovePolicies(String, String, Vec<Vec<String>>),
    RemoveFilteredPolicy(String, String, Vec<Vec<String>>),
}

pub struct SyncedEnforcer {
    enforcer: Enforcer,
    events: HashMap<Event, Vec<EventCallback>>,
    autoload_running: bool,
    metrics: Option<Arc<KustosMetrics>>,
}

impl SyncedEnforcer {
    pub fn is_autoload_running(&self) -> bool {
        self.autoload_running
    }

    #[tracing::instrument(level = "trace", skip(self, user))]
    pub fn get_implicit_resources_for_user<U: Into<PolicyUser>>(
        &mut self,
        user: U,
    ) -> crate::Result<Vec<UserPolicy>> {
        let policy_user: PolicyUser = user.into();
        self.enforcer
            .get_implicit_resources_for_user(&policy_user.to_casbin_string(), None)
            .into_iter()
            .map(TryInto::<UserPolicy>::try_into)
            .map(|r| r.map_err(Into::into))
            .collect()
    }

    /// Checks if a casbin policy of type p is present
    pub fn has_policy<P: ToCasbin>(&self, policy: P) -> bool {
        self.enforcer.has_policy(policy.to_casbin_policy())
    }

    // Checks if a casbin policy of type g is present
    pub fn has_group_policy<P: ToCasbin>(&mut self, policy: P) -> bool {
        let policy = policy.to_casbin_policy();
        self.enforcer
            .has_role_for_user(&policy[0], &policy[1], None)
    }

    pub fn set_metrics(&mut self, metrics: Arc<KustosMetrics>) {
        self.metrics = Some(metrics)
    }

    /// The autoloader creates the need for using the enforcer inside of a tokio::Mutex
    pub async fn start_autoload_policy(
        enforcer: Arc<RwLock<SyncedEnforcer>>,
        rabbitmq_pool: Arc<RabbitMqPool>,
    ) -> crate::Result<()> {
        let mut write = enforcer.write().await;

        if write.autoload_running {
            return Err(crate::Error::AutoloadRunning);
        }

        write.autoload_running = true;
        drop(write);

        tokio::spawn(sync_task(enforcer, rabbitmq_pool));

        Ok(())
    }

    async fn apply(&mut self, event_data: KustosEventData) -> CasbinResult<()> {
        // Disable auto save here to avoid writing to the database since this is already written by the sending controller
        self.enforcer.enable_auto_save(false);

        let result = match event_data {
            KustosEventData::AddPolicy(sec, ptype, params) => {
                self.enforcer
                    .add_policy_internal(&sec, &ptype, params)
                    .await
            }
            KustosEventData::AddPolicies(sec, ptype, params) => {
                self.enforcer
                    .add_policies_internal(&sec, &ptype, params)
                    .await
            }
            KustosEventData::RemovePolicy(sec, ptype, rule) => {
                self.enforcer
                    .remove_policy_internal(&sec, &ptype, rule)
                    .await
            }
            KustosEventData::RemovePolicies(sec, ptype, rules) => {
                self.enforcer
                    .remove_policies_internal(&sec, &ptype, rules)
                    .await
            }
            KustosEventData::RemoveFilteredPolicy(sec, ptype, policy) => {
                self.enforcer
                    .remove_policies_internal(&sec, &ptype, policy)
                    .await
            }
        };

        self.enforcer.enable_auto_save(true);

        result.map(|_| ())
    }
}

async fn sync_task(
    enforcer: Arc<RwLock<SyncedEnforcer>>,
    rabbitmq_pool: Arc<RabbitMqPool>,
) -> Result<(), ()> {
    let id = ShortString::from(Uuid::new_v4().to_string());

    loop {
        if let Err(e) = synchronize_until_error(&rabbitmq_pool, &id, &enforcer).await {
            log::error!("synchronization failed with error: {e}");
        }

        // Unregister event handler
        enforcer.write().await.off(Event::PolicyChange);
    }
}

async fn synchronize_until_error(
    rabbitmq_pool: &Arc<RabbitMqPool>,
    id: &ShortString,
    enforcer: &Arc<RwLock<SyncedEnforcer>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let channel = rabbitmq_pool.create_channel().await?;
    let mut consumer = create_rabbitmq_consumer(&channel).await?;

    let mut join_handle =
        register_policy_change_event_handler(id.clone(), enforcer, channel).await?;

    loop {
        tokio::select! {
            _ = &mut join_handle => {
                return Err("send acl update task exited".into());
            },
            delivery = consumer.next() => {
                let Some(delivery) = delivery else {
                    return Err("consumer returned None".into());
                };

                let delivery = delivery?;

                // Don´t handle own messages
                if Some(id) == delivery.properties.correlation_id().as_ref() {
                    continue;
                }

                // Parse message
                let event_data = match serde_json::from_slice::<KustosEventData>(&delivery.data) {
                    Ok(event_data) => event_data,
                    Err(e) => {
                        log::warn!("Failed to parse acl-update event data: {e:?}");
                        continue;
                    }
                };

                enforcer.write().await.apply(event_data).await?;
            }
        }
    }
}

async fn register_policy_change_event_handler(
    id: ShortString,
    enforcer: &Arc<RwLock<SyncedEnforcer>>,
    channel: RabbitMqChannel,
) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
    let mut enforcer = enforcer.write().await;

    // Load the policy once after creating the queue to be up to date with all the other controllers
    enforcer.load_policy().await?;

    let (tx, mut rx) = mpsc::unbounded_channel();

    let join_handle = tokio::spawn({
        let id = id.clone();

        async move {
            while let Some(event_data) = rx.recv().await {
                if let Err(e) = channel
                    .basic_publish(
                        EXCHANGE_NAME,
                        "",
                        BasicPublishOptions::default(),
                        &serde_json::to_vec(&event_data)
                            .expect("KustosEventData must always be serializable"),
                        AMQPProperties::default().with_correlation_id(id.clone()),
                    )
                    .await
                {
                    log::error!("failed to publish acl update: {e:?}");
                    break;
                }
            }
        }
    });

    enforcer.on(
        Event::PolicyChange,
        Box::new(move |_, b| {
            let event_data = match b {
                EventData::AddPolicy(a, b, c) => KustosEventData::AddPolicy(a, b, c),
                EventData::AddPolicies(a, b, c) => KustosEventData::AddPolicies(a, b, c),
                EventData::RemovePolicy(a, b, c) => KustosEventData::RemovePolicy(a, b, c),
                EventData::RemovePolicies(a, b, c) => KustosEventData::RemovePolicies(a, b, c),
                EventData::RemoveFilteredPolicy(a, b, c) => {
                    KustosEventData::RemoveFilteredPolicy(a, b, c)
                }
                EventData::SavePolicy(_) | EventData::ClearPolicy | EventData::ClearCache => {
                    // Not handled
                    return;
                }
            };

            if tx.send(event_data).is_err() {
                log::error!("send event data sync channel dropped");
            }
        }),
    );

    Ok(join_handle)
}

async fn create_rabbitmq_consumer(
    channel: &RabbitMqChannel,
) -> Result<lapin::Consumer, lapin::Error> {
    channel
        .exchange_declare(
            EXCHANGE_NAME,
            ExchangeKind::Fanout,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let queue = channel
        .queue_declare(
            "",
            QueueDeclareOptions {
                exclusive: true,
                auto_delete: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    channel
        .queue_bind(
            queue.name().as_str(),
            EXCHANGE_NAME,
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

    channel
        .basic_consume(
            queue.name().as_str(),
            "",
            BasicConsumeOptions {
                no_ack: true,
                exclusive: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
}

impl EventEmitter<Event> for SyncedEnforcer {
    fn on(&mut self, e: Event, f: Box<dyn FnMut(&mut Self, EventData) + Send + Sync>) {
        self.events.entry(e).or_default().push(f)
    }

    fn off(&mut self, e: Event) {
        self.events.remove(&e);
    }

    fn emit(&mut self, e: Event, d: EventData) {
        let mut events = take(&mut self.events);

        if let Some(cbs) = events.get_mut(&e) {
            for cb in cbs.iter_mut() {
                cb(self, d.clone())
            }
        }

        self.events = events;
    }
}

#[async_trait]
impl CoreApi for SyncedEnforcer {
    async fn new_raw<M, A>(m: M, a: A) -> CasbinResult<SyncedEnforcer>
    where
        M: TryIntoModel,
        A: TryIntoAdapter,
    {
        let mut enforcer = Enforcer::new_raw(m, a).await?;

        enforcer.add_function("actMatch", |req, pol| {
            super::custom_matcher::act_match(&req, &pol)
        });
        enforcer.add_function("objMatch", |req, pol| {
            super::custom_matcher::obj_match(&req, &pol)
        });

        let enforcer = SyncedEnforcer {
            enforcer,
            events: HashMap::new(),
            autoload_running: false,
            metrics: None,
        };

        Ok(enforcer)
    }

    #[inline]
    async fn new<M, A>(m: M, a: A) -> CasbinResult<SyncedEnforcer>
    where
        M: TryIntoModel,
        A: TryIntoAdapter,
    {
        let mut enforcer = Self::new_raw(m, a).await?;
        enforcer.load_policy().await?;
        Ok(enforcer)
    }

    #[inline]
    fn add_function(&mut self, fname: &str, f: fn(ImmutableString, ImmutableString) -> bool) {
        self.enforcer.add_function(fname, f);
    }

    #[inline]
    #[tracing::instrument(level = "trace", skip(self))]
    fn get_model(&self) -> &dyn Model {
        self.enforcer.get_model()
    }

    #[inline]
    #[tracing::instrument(level = "trace", skip(self))]
    fn get_mut_model(&mut self) -> &mut dyn Model {
        self.enforcer.get_mut_model()
    }

    #[inline]
    fn get_adapter(&self) -> &dyn Adapter {
        self.enforcer.get_adapter()
    }

    #[inline]
    fn get_mut_adapter(&mut self) -> &mut dyn Adapter {
        self.enforcer.get_mut_adapter()
    }

    #[inline]
    fn get_role_manager(&self) -> Arc<parking_lot::RwLock<dyn RoleManager>> {
        self.enforcer.get_role_manager()
    }

    #[inline]
    fn get_role_managers(&self) -> Box<dyn Iterator<Item = Arc<pl::RwLock<dyn RoleManager>>> + '_> {
        self.enforcer.get_role_managers()
    }

    #[inline]
    fn get_role_manager_for_ptype(&self, ptype: &str) -> Option<Arc<pl::RwLock<dyn RoleManager>>> {
        self.enforcer.get_role_manager_for_ptype(ptype)
    }

    #[inline]
    fn set_role_manager(
        &mut self,
        rm: Arc<parking_lot::RwLock<dyn RoleManager>>,
    ) -> CasbinResult<()> {
        self.enforcer.set_role_manager(rm)
    }

    #[inline]
    async fn set_model<M: TryIntoModel>(&mut self, m: M) -> CasbinResult<()> {
        self.enforcer.set_model(m).await
    }

    #[inline]
    async fn set_adapter<A: TryIntoAdapter>(&mut self, a: A) -> CasbinResult<()> {
        self.enforcer.set_adapter(a).await
    }

    #[inline]
    fn set_effector(&mut self, e: Box<dyn Effector>) {
        self.enforcer.set_effector(e);
    }

    #[inline]
    #[tracing::instrument(level = "trace", skip(self, rvals))]
    fn enforce<ARGS: EnforceArgs>(&self, rvals: ARGS) -> CasbinResult<bool> {
        let rm = self.get_role_manager();
        let rm = rm.read();

        let mut rvals = rvals.try_into_vec()?;

        // Fetch all policies in casbin
        let model = self.get_model().get_model();

        let request_model = &model["r"]["r"];
        let policy_model = &model["p"]["p"];

        // Assert the enforcer's model matches what this function does
        assert_eq!(&request_model.tokens, &["r_sub", "r_obj", "r_act"]);
        assert_eq!(&policy_model.tokens, &["p_sub", "p_obj", "p_act"]);

        let policies = policy_model.get_policy();

        // kustos's permission model expects 3 fields per policy: sub, obj, act
        // So make sure that the input is 3 values.
        if rvals.len() != 3 {
            return Err(casbin::Error::PolicyError(
                casbin::error::PolicyError::UnmatchPolicyDefinition(3, rvals.len()),
            ));
        }

        // Only way to make the `rvals` generic into strings is to cast them to rhai strings.
        let act = rvals.pop().unwrap().into_immutable_string().unwrap();
        let obj = rvals.pop().unwrap().into_immutable_string().unwrap();
        let sub = rvals.pop().unwrap().into_immutable_string().unwrap();

        // This is the matcher function from the kustos model in more efficient order
        // and without checking for OPTIONS act
        //
        // m = g(r.sub, p.sub) && objMatch(r.obj, p.obj) && actMatch(r.act, p.act)
        for policy in policies {
            use super::custom_matcher::*;

            if !obj_match(&obj, &policy[1]) || !act_match(&act, &policy[2]) {
                continue;
            }

            // g function (user role/group relation check)
            //
            // Role manager checks if the checking subject is related to the subject in this policy
            //
            // e.g. imagine following policies
            //
            // "g, user::bob, role::admin"       # defines `user::bob` as having the role 'admin'
            // "p, role::admin, myresource, GET" # Everyone with the role admin can GET `myresource`
            //
            // When checking if `user::bob` can access `myresource`;
            // `rm.has_link("user::bob", "role::admin")` will return true,
            // so `user::bob` will be granted access to to `myresource`.
            if !rm.has_link(&sub, &policy[0], None) {
                continue;
            }

            return Ok(true);
        }

        Ok(false)
    }

    #[inline]
    #[tracing::instrument(level = "trace", skip(self, rvals))]
    fn enforce_mut<ARGS: EnforceArgs>(&mut self, rvals: ARGS) -> CasbinResult<bool> {
        let start = Instant::now();

        let res = self.enforcer.enforce_mut(rvals)?;

        if let Some(metrics) = &self.metrics {
            metrics
                .enforce_execution_time
                .record(start.elapsed().as_secs_f64(), &[]);
        }

        Ok(res)
    }

    #[inline]
    #[tracing::instrument(level = "trace", skip(self))]
    fn build_role_links(&mut self) -> CasbinResult<()> {
        self.enforcer.build_role_links()
    }

    #[inline]
    #[tracing::instrument(level = "trace", skip(self, d), fields(event_data = %d))]
    fn build_incremental_role_links(&mut self, d: EventData) -> CasbinResult<()> {
        self.enforcer.build_incremental_role_links(d)
    }

    #[inline]
    async fn load_policy(&mut self) -> CasbinResult<()> {
        let start = Instant::now();

        self.enforcer.load_policy().await?;

        if let Some(metrics) = &self.metrics {
            metrics
                .load_policy_execution_time
                .record(start.elapsed().as_secs_f64(), &[]);
        }

        Ok(())
    }

    #[inline]
    async fn load_filtered_policy<'a>(&mut self, f: Filter<'a>) -> CasbinResult<()> {
        self.enforcer.load_filtered_policy(f).await
    }

    #[inline]
    fn is_filtered(&self) -> bool {
        self.enforcer.is_filtered()
    }

    #[inline]
    fn is_enabled(&self) -> bool {
        self.enforcer.is_enabled()
    }

    #[inline]
    #[tracing::instrument(level = "trace", skip(self))]
    async fn save_policy(&mut self) -> CasbinResult<()> {
        self.enforcer.save_policy().await
    }

    #[inline]
    async fn clear_policy(&mut self) -> CasbinResult<()> {
        self.enforcer.clear_policy().await
    }

    #[inline]
    fn enable_auto_save(&mut self, auto_save: bool) {
        self.enforcer.enable_auto_save(auto_save);
    }

    #[inline]
    fn enable_enforce(&mut self, enabled: bool) {
        self.enforcer.enable_enforce(enabled);
    }

    #[inline]
    fn enable_auto_build_role_links(&mut self, auto_build_role_links: bool) {
        self.enforcer
            .enable_auto_build_role_links(auto_build_role_links);
    }

    #[inline]
    fn has_auto_save_enabled(&self) -> bool {
        self.enforcer.has_auto_save_enabled()
    }

    fn set_watcher(&mut self, w: Box<dyn casbin::Watcher>) {
        self.enforcer.set_watcher(w);
    }
    fn get_watcher(&self) -> Option<&dyn casbin::Watcher> {
        self.enforcer.get_watcher()
    }
    fn get_mut_watcher(&mut self) -> Option<&mut dyn casbin::Watcher> {
        self.enforcer.get_mut_watcher()
    }
    fn enable_auto_notify_watcher(&mut self, auto_notify_watcher: bool) {
        self.enforcer
            .enable_auto_notify_watcher(auto_notify_watcher);
    }
    fn has_auto_notify_watcher_enabled(&self) -> bool {
        self.enforcer.has_auto_notify_watcher_enabled()
    }

    #[inline]
    fn has_auto_build_role_links_enabled(&self) -> bool {
        self.enforcer.has_auto_build_role_links_enabled()
    }
}

#[cfg(test)]
mod tests {
    use casbin::{CoreApi, MgmtApi};

    use super::SyncedEnforcer;
    use crate::internal::default_acl_model;

    fn to_owned(v: Vec<&str>) -> Vec<String> {
        v.into_iter().map(|x| x.to_owned()).collect()
    }

    fn to_owned2(v: Vec<Vec<&str>>) -> Vec<Vec<String>> {
        v.into_iter().map(to_owned).collect()
    }

    #[tokio::test]
    async fn enforce() {
        let m = default_acl_model().await;
        let a = casbin::MemoryAdapter::default();
        let mut enforcer = SyncedEnforcer::new(m, a).await.unwrap();

        let policies = vec![
            vec!["role::user", "/rooms", "POST|GET"],
            vec!["user::bob", "/rooms", "UPDATE"],
            vec!["user::bob", "/events/*", "GET|POST"],
        ];

        enforcer.add_policies(to_owned2(policies)).await.unwrap();

        enforcer
            .add_grouping_policy(to_owned(vec!["user::bob", "role::user"]))
            .await
            .unwrap();

        assert!(enforcer
            .enforce(to_owned(vec!["user::bob", "/rooms", "GET"]))
            .unwrap());
        assert!(enforcer
            .enforce(to_owned(vec!["user::bob", "/rooms", "POST"]))
            .unwrap());
        assert!(!enforcer
            .enforce(to_owned(vec!["user::bob", "/rooms", "DELETE"]))
            .unwrap());

        assert!(enforcer
            .enforce(to_owned(vec!["user::bob", "/rooms", "UPDATE"]))
            .unwrap());

        assert!(!enforcer
            .enforce(to_owned(vec!["user::bob", "/rooms/abc", "UPDATE"]))
            .unwrap());

        assert!(enforcer
            .enforce(to_owned(vec!["user::bob", "/events/1", "GET"]))
            .unwrap());
        assert!(enforcer
            .enforce(to_owned(vec!["user::bob", "/events/1", "POST"]))
            .unwrap());

        assert!(!enforcer
            .enforce(to_owned(vec!["user::bob", "/events/1", "UPDATE"]))
            .unwrap());
    }
}
