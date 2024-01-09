// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Provides various internal implementations

use casbin::{rhai::Dynamic, EnforceArgs};
use itertools::Itertools;

use std::{
    collections::hash_map::DefaultHasher,
    convert::TryFrom,
    hash::{Hash, Hasher},
    str::FromStr,
};

use super::{ToCasbin, ToCasbinMultiple, ToCasbinString};
use crate::{
    access::AccessMethod,
    error::ParsingError,
    policy::{InvitePolicy, Policies, Policy, UserPolicy},
    resource::ResourceId,
    subject::{
        GroupToRole, IsSubject, PolicyGroup, PolicyInvite, PolicyRole, PolicyUser, UserToGroup,
        UserToRole,
    },
};

impl ToCasbinString for AccessMethod {
    fn to_casbin_string(self) -> String {
        self.to_string()
    }
}

impl ToCasbinString for &[AccessMethod] {
    /// Converts multiple AccessMethods to a Regex that matches any one of them
    fn to_casbin_string(self) -> String {
        self.iter()
            .map(|&access_method| ToCasbinString::to_casbin_string(access_method))
            .join("|")
    }
}

impl ToCasbinString for PolicyUser {
    fn to_casbin_string(self) -> String {
        format!("user::{}", self.0)
    }
}

impl ToCasbinString for PolicyInvite {
    fn to_casbin_string(self) -> String {
        format!("invite::{}", self.0)
    }
}

impl ToCasbinString for PolicyRole {
    fn to_casbin_string(self) -> String {
        format!("role::{}", self.0)
    }
}

impl ToCasbinString for PolicyGroup {
    fn to_casbin_string(self) -> String {
        format!("group::{}", self.0)
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

impl ToCasbin for GroupToRole {
    fn to_casbin_policy(self) -> Vec<String> {
        vec![self.0.to_casbin_string(), self.1.to_casbin_string()]
    }
}

impl ToCasbin for UserToGroup {
    fn to_casbin_policy(self) -> Vec<String> {
        vec![self.0.to_casbin_string(), self.1.to_casbin_string()]
    }
}

impl ToCasbin for UserToRole {
    fn to_casbin_policy(self) -> Vec<String> {
        vec![self.0.to_casbin_string(), self.1.to_casbin_string()]
    }
}
