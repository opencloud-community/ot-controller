// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Kustos (from latin `custos`, means guard)
//!
//! An authorization provider for use in opentalk.
//! Currently casbin is used as the main library to provide access checks.
//! One benefit over using casbin directly is more typesafety with the added cost of removing some flexibility.
//! This also aims to add an abstraction to be able to switch out the underlying authz framework when needed.
//!
//!
//! This abstraction allows for request-level and resource-level permissions by enforcing a single resource URL (without scheme and authority).
//! For this currently GET, POST, PUT, and DELETE are supported as request-level actions and are also used to enforce resource-level permissions.
//! While the underlying authz framework can use a variance of types Kustos provides strong types for most applications.
//!
//! Subjects:
//! * Users (represented by a UUID)
//! * Groups, user-facing groups (represented by String)
//!   E.g. Groups from a OIDC Provider such as Keycloak
//! * Roles, internal-roles (represented by String)
//!   Like `administrator`, used to provide default super permissions.
//!   The default model allows access to all endpoints for administrators.
//!
//! # Getting Started:
//!
//! 1. Make sure to execute the migrations.
//! 2. Make sure default rules are set. E.g. If `/resource` is a authenticated REST endpoint to add elements to that collection via `POST`,
//!    add the needed rules to allow normal authenticated users to POST to that endpoint to create `resource`s.
//!
//!
//! # Database
//!
//! This crate provides barrel migration files to create the correct database schema for use with the current version.
//! For this call all migrations in the migrations folder in the respective order. You can use a crate like refinery to version your current deployed schema.
//!
//! # Examples:
//!
//! ```rust,no_run
//! # use kustos::prelude::*;
//! # use std::sync::Arc;
//! # use database::Db;
//! # use tokio::runtime::Runtime;
//! # use tokio::task;
//! # use std::time::Duration;
//! # let rt  = Runtime::new().unwrap();
//! # let local = task::LocalSet::new();
//! # local.block_on(&rt, async {
//! let db = Arc::new(
//! Db::connect_url("postgres://postgres:postgres@localhost/kustos", 10).unwrap(),
//! );
//! let (mut shutdown, _) = tokio::sync::broadcast::channel(1);
//! let authz = Authz::new_with_autoload(db, shutdown.subscribe(), Duration::from_secs(10))
//!     .await
//!     .unwrap()
//!     .0;
//! authz
//!     .grant_role_access(
//!         "admin",
//!         &[(&ResourceId::from("/users/*"), &[AccessMethod::POST])],
//!     )
//!     .await;
//! authz
//!     .add_user_to_role(uuid::Uuid::nil(), "admin")
//!     .await;
//! authz
//!     .check_user(
//!         uuid::Uuid::nil(),
//!         "/users/0",
//!         AccessMethod::POST,
//!     )
//!     .await;
//! # });
//! ```
use crate::actix_web::KustosService;
use casbin::{CoreApi, MgmtApi, RbacApi};
use database::Db;
use internal::synced_enforcer::SyncedEnforcer;
use metrics::KustosMetrics;
use policy::{
    GroupPolicies, InvitePolicies, InvitePolicy, Policy, RolePolicies, UserPolicies, UserPolicy,
};
use shared::{
    internal::{ToCasbin, ToCasbinMultiple, ToCasbinString},
    resource::KustosFromStr,
    subject::{
        GroupToRole, IsSubject, PolicyGroup, PolicyInvite, PolicyRole, PolicyUser, UserToGroup,
        UserToRole,
    },
};
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{broadcast::Receiver, RwLock},
    task::JoinHandle,
};

pub mod actix_web;
pub mod db;
mod error;
mod internal;
pub mod metrics;
pub mod policies_builder;
pub mod policy;
pub mod prelude;

pub use error::{Error, Result};
pub use shared::{
    access::AccessMethod,
    error::ResourceParseError,
    resource,
    resource::{AccessibleResources, Resource, ResourceId},
    subject,
};

#[macro_use]
extern crate diesel;

#[derive(Clone)]
pub struct Authz {
    inner: Arc<RwLock<SyncedEnforcer>>,
}

impl Authz {
    pub async fn new(db: Arc<Db>) -> Result<Self> {
        let acl_model = internal::default_acl_model().await;
        let adapter = internal::diesel_adapter::CasbinAdapter::new(db.clone());
        let enforcer = Arc::new(tokio::sync::RwLock::new(
            SyncedEnforcer::new(acl_model, adapter).await?,
        ));

        Ok(Self { inner: enforcer })
    }

    pub async fn new_with_autoload(
        db: Arc<Db>,
        shutdown_channel: Receiver<()>,
        interval: Duration,
    ) -> Result<(Self, JoinHandle<Result<()>>)> {
        let acl_model = internal::default_acl_model().await;
        let adapter = internal::diesel_adapter::CasbinAdapter::new(db.clone());
        let enforcer = Arc::new(tokio::sync::RwLock::new(
            SyncedEnforcer::new(acl_model, adapter).await?,
        ));

        let handle =
            SyncedEnforcer::start_autoload_policy(enforcer.clone(), interval, shutdown_channel)
                .await?;

        Ok((Self { inner: enforcer }, handle))
    }

    pub async fn new_with_autoload_and_metrics(
        db: Arc<Db>,
        shutdown_channel: Receiver<()>,
        interval: Duration,
        metrics: Arc<KustosMetrics>,
    ) -> Result<(Self, JoinHandle<Result<()>>)> {
        let acl_model = internal::default_acl_model().await;
        let adapter = internal::diesel_adapter::CasbinAdapter::new(db.clone());
        let mut enforcer = SyncedEnforcer::new(acl_model, adapter).await?;
        enforcer.set_metrics(metrics);

        let enforcer = Arc::new(tokio::sync::RwLock::new(enforcer));

        let handle =
            SyncedEnforcer::start_autoload_policy(enforcer.clone(), interval, shutdown_channel)
                .await?;

        Ok((Self { inner: enforcer }, handle))
    }

    // TODO(r.floren) would be place for a feature gate to use this with other frameworks as well.
    pub async fn actix_web_middleware(&self, strip_versioned_path: bool) -> Result<KustosService> {
        Ok(crate::actix_web::KustosService::new(self.inner.clone(), strip_versioned_path).await?)
    }

    /// Grants the user access to the resources with access
    ///
    /// This takes multiple resources with access at once.
    #[tracing::instrument(level = "debug", skip(self, user, data_access))]
    pub async fn grant_user_access<'a, T>(
        &self,
        user: T,
        data_access: &'a [(&'a ResourceId, &'a [AccessMethod])],
    ) -> Result<()>
    where
        T: Into<PolicyUser>,
    {
        let user = user.into();
        let policies = UserPolicies::new(&user, data_access);

        // The return value of casbins add_policy means: true => newly added, false => already in model.
        // Thus it is sane to just listen for errors here.
        self.inner
            .write()
            .await
            .add_policies(policies.to_casbin_policies())
            .await?;
        Ok(())
    }

    /// Grants the user access to the resources with access
    ///
    /// This takes multiple resources with access at once.
    #[tracing::instrument(level = "debug", skip(self, invite, data_access))]
    pub async fn grant_invite_access<'a, T>(
        &self,
        invite: T,
        data_access: &'a [(&'a ResourceId, &'a [AccessMethod])],
    ) -> Result<()>
    where
        T: Into<PolicyInvite>,
    {
        let invite = invite.into();
        let policies = InvitePolicies::new(&invite, data_access);

        // The return value of casbins add_policy means: true => newly added, false => already in model.
        // Thus it is sane to just listen for errors here.
        self.inner
            .write()
            .await
            .add_policies(policies.to_casbin_policies())
            .await?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn add_policies(&self, policies: impl ToCasbinMultiple) -> Result<()> {
        // Applying the policies in chunks in order to avoid generating too large queries
        for chunk in policies.to_casbin_policies().chunks(2000) {
            self.inner
                .write()
                .await
                .add_policies(Vec::from(chunk))
                .await?;
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn remove_policies(&self, policies: impl ToCasbinMultiple) -> Result<()> {
        self.inner
            .write()
            .await
            .remove_policies(policies.to_casbin_policies())
            .await?;

        Ok(())
    }

    /// Grants the group access to the resources with access
    ///
    /// This takes multiple resources with access at once.
    #[tracing::instrument(level = "debug", skip(self, group, data_access))]
    pub async fn grant_group_access<'a, T>(
        &self,
        group: T,
        data_access: &'a [(&'a ResourceId, &'a [AccessMethod])],
    ) -> Result<()>
    where
        T: Into<PolicyGroup>,
    {
        let group = group.into();
        let policies = GroupPolicies::new(&group, data_access);

        self.inner
            .write()
            .await
            .add_policies(policies.to_casbin_policies())
            .await?;
        Ok(())
    }

    /// Grants the role access to the resources with access
    ///
    /// This takes multiple resources with access at once.
    #[tracing::instrument(level = "debug", skip(self, role, data_access))]
    pub async fn grant_role_access<'a, T>(
        &self,
        role: T,
        data_access: &'a [(&'a ResourceId, &'a [AccessMethod])],
    ) -> Result<()>
    where
        T: Into<PolicyRole>,
    {
        let role = role.into();
        let policies = RolePolicies::new(&role, data_access);

        self.inner
            .write()
            .await
            .add_policies(policies.to_casbin_policies())
            .await?;
        Ok(())
    }

    /// Adds user to group
    #[tracing::instrument(level = "debug", skip(self, user, group))]
    pub async fn add_user_to_group<T, B>(&self, user: T, group: B) -> Result<()>
    where
        T: Into<PolicyUser>,
        B: Into<PolicyGroup>,
    {
        self.inner
            .write()
            .await
            .add_role_for_user(
                &user.into().to_casbin_string(),
                &group.into().to_casbin_string(),
                None,
            )
            .await?;
        Ok(())
    }

    /// Removes user from group
    #[tracing::instrument(level = "debug", skip(self, user, group))]
    pub async fn remove_user_from_group<T, B>(&self, user: T, group: B) -> Result<()>
    where
        T: Into<PolicyUser>,
        B: Into<PolicyGroup>,
    {
        self.inner
            .write()
            .await
            .delete_role_for_user(
                &user.into().to_casbin_string(),
                &group.into().to_casbin_string(),
                None,
            )
            .await?;
        Ok(())
    }

    /// Add user to role
    #[tracing::instrument(level = "debug", skip(self, user, role))]
    pub async fn add_user_to_role<T, B>(&self, user: T, role: B) -> Result<()>
    where
        T: Into<PolicyUser>,
        B: Into<PolicyRole>,
    {
        self.inner
            .write()
            .await
            .add_role_for_user(
                &user.into().to_casbin_string(),
                &role.into().to_casbin_string(),
                None,
            )
            .await?;
        Ok(())
    }

    /// Remove user from role
    #[tracing::instrument(level = "debug", skip(self, user, role))]
    pub async fn remove_user_from_role<T, B>(&self, user: T, role: B) -> Result<()>
    where
        T: Into<PolicyUser>,
        B: Into<PolicyRole>,
    {
        self.inner
            .write()
            .await
            .delete_role_for_user(
                &user.into().to_casbin_string(),
                &role.into().to_casbin_string(),
                None,
            )
            .await?;
        Ok(())
    }

    /// Adds the group to role
    #[tracing::instrument(level = "debug", skip(self, group, role))]
    pub async fn add_group_to_role<T, B>(&self, group: T, role: B) -> Result<()>
    where
        T: Into<PolicyGroup>,
        B: Into<PolicyRole>,
    {
        self.inner
            .write()
            .await
            .add_role_for_user(
                &group.into().to_casbin_string(),
                &role.into().to_casbin_string(),
                None,
            )
            .await?;
        Ok(())
    }

    /// Removes the group from role
    #[tracing::instrument(level = "debug", skip(self, group, role))]
    pub async fn remove_group_from_role<T, B>(&self, group: T, role: B) -> Result<()>
    where
        T: Into<PolicyGroup>,
        B: Into<PolicyRole>,
    {
        self.inner
            .write()
            .await
            .delete_role_for_user(
                &group.into().to_casbin_string(),
                &role.into().to_casbin_string(),
                None,
            )
            .await?;
        Ok(())
    }

    /// Checks if the given user has access to the resource
    #[tracing::instrument(level = "debug", skip(self, user, res, access))]
    pub async fn check_user<U, R>(&self, user: U, res: R, access: AccessMethod) -> Result<bool>
    where
        U: Into<PolicyUser>,
        R: Into<ResourceId>,
    {
        Ok(self
            .inner
            .read()
            .await
            .enforce(UserPolicy::new(user.into(), res, access))?)
    }

    /// Checks if the given invite code has access to the resource
    #[tracing::instrument(level = "debug", skip(self, invite, res, access))]
    pub async fn check_invite<I, R>(&self, invite: I, res: R, access: AccessMethod) -> Result<bool>
    where
        I: Into<PolicyInvite>,
        R: Into<ResourceId>,
    {
        Ok(self
            .inner
            .read()
            .await
            .enforce(InvitePolicy::new(invite.into(), res, access))?)
    }

    /// Checks in a batched way if the given user has access to the resources
    #[tracing::instrument(level = "debug", skip(self, user, res, access))]
    pub async fn check_user_batched<U, R, I>(
        &self,
        user: U,
        res: I,
        access: AccessMethod,
    ) -> Result<Vec<bool>>
    where
        U: Into<PolicyUser> + Clone,
        R: Into<ResourceId>,
        I: IntoIterator<Item = R>,
    {
        let inner = self.inner.read().await;
        let iterator = res.into_iter();
        let mut result = Vec::with_capacity(iterator.size_hint().0);
        for resource in iterator {
            result.push(inner.enforce(UserPolicy::new(user.clone().into(), resource, access))?);
        }
        Ok(result)
    }

    /// Returns the accessible resource for the given user.
    #[tracing::instrument(level = "debug", skip(self, user, access))]
    pub async fn get_accessible_resources_for_user<U, T>(
        &self,
        user: U,
        access: AccessMethod,
    ) -> Result<AccessibleResources<T>>
    where
        U: Into<PolicyUser>,
        T: Resource + KustosFromStr,
    {
        let policies = self
            .inner
            .write()
            .await
            .get_implicit_resources_for_user(user)?;

        let prefix = T::PREFIX;
        let type_wildcard = format!("{prefix}*");

        // Check if a wildcard policy is present and fulfills the access request
        let has_wildcard_access = policies.iter().find(|x| {
            (x.obj().as_str() == type_wildcard || x.obj().as_str() == "/*")
                && x.act().contains(&access)
        });

        if has_wildcard_access.is_some() {
            return Ok(AccessibleResources::All);
        }

        // Transform the list of accessible resources from strings to T
        let resources = policies
            .into_iter()
            .filter(|x| x.act().contains(&access))
            .map(|x| -> ResourceId { x.into() })
            .filter_map(|x| {
                x.strip_prefix(prefix)
                    // Filter everything that still contains a slash, these are subresources
                    .filter(|x| !x.contains('/'))
                    .map(KustosFromStr::kustos_from_str)
            })
            .collect::<Result<Vec<T>, ResourceParseError>>()?;

        Ok(AccessibleResources::List(resources))
    }

    /// Checks if the given permissions is present
    #[tracing::instrument(level = "debug", skip(self, subject, res, access))]
    pub async fn is_permissions_present<T, R, A>(
        &self,
        subject: T,
        res: R,
        access: A,
    ) -> Result<bool>
    where
        T: IsSubject + ToCasbinString,
        R: Into<ResourceId>,
        A: Into<Vec<AccessMethod>>,
    {
        Ok(self
            .inner
            .read()
            .await
            .has_policy(Policy::<T>::new(subject, res, access)))
    }

    /// Checks if the given group is in the given role
    #[tracing::instrument(level = "debug", skip(self, group, role))]
    pub async fn is_group_in_role<G, R>(&self, group: G, role: R) -> Result<bool>
    where
        G: Into<PolicyGroup>,
        R: Into<PolicyRole>,
    {
        Ok(self
            .inner
            .write()
            .await
            .has_group_policy(GroupToRole(group.into(), role.into())))
    }

    /// Checks if the given user is in the given group
    #[tracing::instrument(level = "debug", skip(self, user, group))]
    pub async fn is_user_in_group<G, R>(&self, user: G, group: R) -> Result<bool>
    where
        G: Into<PolicyUser>,
        R: Into<PolicyGroup>,
    {
        Ok(self
            .inner
            .write()
            .await
            .has_group_policy(UserToGroup(user.into(), group.into())))
    }

    /// Checks if the given user is in the given role
    #[tracing::instrument(level = "debug", skip(self, user, role))]
    pub async fn is_user_in_role<G, R>(&self, user: G, role: R) -> Result<bool>
    where
        G: Into<PolicyUser>,
        R: Into<PolicyRole>,
    {
        Ok(self
            .inner
            .write()
            .await
            .has_group_policy(UserToRole(user.into(), role.into())))
    }

    /// Removes all rules that explicitly name the given resource
    #[tracing::instrument(level = "debug", skip(self, resource))]
    pub async fn remove_explicit_resource_permissions<R>(&self, resource: R) -> Result<bool>
    where
        R: Into<ResourceId>,
    {
        self.inner
            .write()
            .await
            .remove_filtered_policy(1, vec![resource.into().to_string()])
            .await
            .map_err(Into::into)
    }

    /// Removes all rules that explicitly name the given resource
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn remove_explicit_resources<I, R>(&self, resources: I) -> Result<usize>
    where
        I: IntoIterator<Item = R>,
        R: Into<ResourceId>,
    {
        let mut inner = self.inner.write().await;

        let mut amount = 0;

        for resource in resources {
            let removed = inner
                .remove_filtered_policy(1, vec![resource.into().to_string()])
                .await?;

            if removed {
                amount += 1;
            }
        }

        Ok(amount)
    }

    /// Removes access for user
    #[tracing::instrument(level = "debug", skip(self, user, resource, access))]
    pub async fn remove_user_permission<S, R, A>(
        &self,
        user: S,
        resource: R,
        access: A,
    ) -> Result<bool>
    where
        S: Into<PolicyUser>,
        R: Into<ResourceId>,
        A: Into<Vec<AccessMethod>>,
    {
        self.inner
            .write()
            .await
            .remove_policy(
                Policy::<PolicyUser>::new(user.into(), resource.into(), access.into())
                    .to_casbin_policy(),
            )
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn remove_all_user_permission_for_resources<S, R, I>(
        &self,
        user: S,
        resources: I,
    ) -> Result<usize>
    where
        S: Into<PolicyUser>,
        R: Into<ResourceId>,
        I: IntoIterator<Item = R>,
    {
        let mut inner = self.inner.write().await;

        let user = user.into();
        let user = user.to_casbin_string();

        let mut amount = 0;

        for resource in resources {
            let removed = inner
                .remove_filtered_policy(0, vec![user.clone(), resource.into().to_string()])
                .await?;

            if removed {
                amount += 1;
            }
        }

        Ok(amount)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn remove_all_invite_permission_for_resources<S, R, I>(
        &self,
        invite: S,
        resources: I,
    ) -> Result<usize>
    where
        S: Into<PolicyInvite>,
        R: Into<ResourceId>,
        I: IntoIterator<Item = R>,
    {
        let mut inner = self.inner.write().await;

        let invite = invite.into();
        let invite = invite.to_casbin_string();

        let mut amount = 0;

        for resource in resources {
            let removed = inner
                .remove_filtered_policy(0, vec![invite.clone(), resource.into().to_string()])
                .await?;

            if removed {
                amount += 1;
            }
        }

        Ok(amount)
    }

    /// Removes access for role
    #[tracing::instrument(level = "debug", skip(self, user, resource, access))]
    pub async fn remove_role_permission<S, R, A>(
        &self,
        user: S,
        resource: R,
        access: A,
    ) -> Result<bool>
    where
        S: Into<PolicyRole>,
        R: Into<ResourceId>,
        A: Into<Vec<AccessMethod>>,
    {
        self.inner
            .write()
            .await
            .remove_policy(
                Policy::<PolicyRole>::new(user.into(), resource.into(), access.into())
                    .to_casbin_policy(),
            )
            .await
            .map_err(Into::into)
    }

    pub async fn clear_all_policies(&self) -> Result<()> {
        let mut inner = self.inner.write().await;

        inner.clear_policy().await?;

        // clear_policy doesn't seem to update the role managers, so doing it manually here
        inner.get_role_managers().for_each(|rm| rm.write().clear());

        Ok(())
    }
}
