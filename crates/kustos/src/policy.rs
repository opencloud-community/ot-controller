// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    str::FromStr,
};

use casbin::{rhai::Dynamic, EnforceArgs};
use shared::{
    access::AccessMethod,
    error::ParsingError,
    internal::{ToCasbin, ToCasbinMultiple, ToCasbinString},
};

use crate::{
    resource::ResourceId,
    subject::{IsSubject, PolicyGroup, PolicyInvite, PolicyRole, PolicyUser},
};

/// A policy
///
/// Has a subject, an object, and an action.
/// Can be read as subject can do action on object.
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Policy<S: IsSubject> {
    pub(crate) sub: S,
    pub(crate) obj: ResourceId,
    pub(crate) act: Vec<AccessMethod>,
}

impl<S: IsSubject> Policy<S> {
    pub fn new<U, D, A>(subject: U, data: D, access: A) -> Self
    where
        U: Into<S>,
        D: Into<ResourceId>,
        A: Into<Vec<AccessMethod>>,
    {
        Self {
            sub: subject.into(),
            obj: data.into(),
            act: access.into(),
        }
    }

    pub fn sub(&self) -> &S {
        &self.sub
    }

    pub fn obj(&self) -> &ResourceId {
        &self.obj
    }

    pub fn act(&self) -> &[AccessMethod] {
        &self.act
    }
}

/// Short version type alias for a Policy with a PolicyUser subject
pub type UserPolicy = Policy<PolicyUser>;

/// Short version type alias for a Policy with a PolicyInvite subject
pub type InvitePolicy = Policy<PolicyInvite>;

/// Short version type alias for a Policy with a PolicyGroup subject
pub type GroupPolicy = Policy<PolicyGroup>;

/// Short version type alias for a Policy with a PolicyRole subject
pub type RolePolicy = Policy<PolicyRole>;

/// Helper to reduce the amount of boilerplate you need when setting up new permissions
pub struct Policies<'a, T: IsSubject> {
    pub(crate) sub: &'a T,
    pub(crate) data_access: &'a [(&'a ResourceId, &'a [AccessMethod])],
}

pub type UserPolicies<'a> = Policies<'a, PolicyUser>;
pub type InvitePolicies<'a> = Policies<'a, PolicyInvite>;
pub type GroupPolicies<'a> = Policies<'a, PolicyGroup>;
pub type RolePolicies<'a> = Policies<'a, PolicyRole>;

impl<'a, T: IsSubject> Policies<'a, T> {
    pub fn new(sub: &'a T, data_access: &'a [(&'a ResourceId, &'a [AccessMethod])]) -> Self {
        Self { sub, data_access }
    }
}

impl<T: IsSubject + Clone> From<Policies<'_, T>> for Vec<Policy<T>> {
    fn from(policies: Policies<'_, T>) -> Self {
        policies
            .data_access
            .iter()
            .map(|(obj, acts)| {
                let sub = policies.sub.to_owned();
                let obj: ResourceId = (*obj).clone();
                Policy::<T>::new(sub, obj, *acts)
            })
            .collect()
    }
}

impl<T: FromStr<Err = ParsingError> + IsSubject> TryFrom<Vec<String>> for Policy<T> {
    type Error = ParsingError;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        if value.len() != 3 {
            return Err(ParsingError::Custom(format!(
                "Wrong amount of casbin args: {value:?}",
            )));
        }

        Ok(Policy {
            sub: T::from_str(&value[0])?,
            obj: ResourceId::from(value[1].as_str()),
            act: value[2]
                .split('|')
                .map(AccessMethod::from_str)
                .collect::<Result<Vec<AccessMethod>, _>>()?,
        })
    }
}

impl<S: IsSubject + ToCasbinString> ToCasbin for Policy<S> {
    fn to_casbin_policy(self) -> Vec<String> {
        vec![
            self.sub.to_casbin_string(),
            self.obj.into_inner(),
            self.act.to_casbin_string(),
        ]
    }
}

impl<S: IsSubject + ToCasbinString + Clone> From<Policies<'_, S>> for Vec<Vec<String>> {
    fn from(val: Policies<'_, S>) -> Self {
        Into::<Vec<Policy<S>>>::into(val)
            .into_iter()
            .map(Into::<Policy<S>>::into)
            .map(|policy| policy.to_casbin_policy())
            .collect()
    }
}

impl<S: IsSubject + ToCasbinString + Clone> ToCasbinMultiple for Policies<'_, S> {
    fn to_casbin_policies(self) -> Vec<Vec<String>> {
        self.into()
    }
}

impl EnforceArgs for UserPolicy {
    fn try_into_vec(self) -> Result<Vec<Dynamic>, casbin::Error> {
        Ok(vec![
            Dynamic::from(self.sub.to_casbin_string()),
            Dynamic::from(self.obj.into_inner()),
            Dynamic::from(self.act.to_casbin_string()),
        ])
    }

    fn cache_key(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl EnforceArgs for InvitePolicy {
    fn try_into_vec(self) -> Result<Vec<Dynamic>, casbin::Error> {
        Ok(vec![
            Dynamic::from(self.sub.to_casbin_string()),
            Dynamic::from(self.obj.into_inner()),
            Dynamic::from(self.act.to_casbin_string()),
        ])
    }

    fn cache_key(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl<T: IsSubject> From<Policy<T>> for ResourceId {
    fn from(policy: Policy<T>) -> Self {
        policy.obj
    }
}
