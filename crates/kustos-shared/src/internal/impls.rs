// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Provides various internal implementations

use itertools::Itertools;

use super::{ToCasbin, ToCasbinString};
use crate::{
    access::AccessMethod,
    subject::{
        GroupToRole, PolicyGroup, PolicyInvite, PolicyRole, PolicyUser, UserToGroup, UserToRole,
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
