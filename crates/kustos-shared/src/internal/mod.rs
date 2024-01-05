// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

pub mod impls;

pub trait ToCasbin {
    fn to_casbin_policy(self) -> Vec<String>;
}

//TODO(r.floren) find better name for this. We do not want that to be part of ToCasbin as then we need to impl to_casbin_policy ofr UserPolicies<'_>
pub trait ToCasbinMultiple {
    fn to_casbin_policies(self) -> Vec<Vec<String>>;
}

/// This trait is used to allow different struct to be used with casbin.
///
/// This allows strict rules for keys in the permissions system.
/// Similar to redis we use prefixes here to differentiate resources.
/// E.g. `user::<UUID>` and `group::<ID>`
pub trait ToCasbinString {
    fn to_casbin_string(self) -> String;
}
