// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! OpenTalk Controller service
//!
//! This crate contains the default OpenTalk Controller backend implementation.

#![deny(
    bad_style,
    missing_debug_implementations,
    missing_docs,
    overflowing_literals,
    patterns_in_fns_without_body,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]

mod controller_backend;

pub use controller_backend::ControllerBackend;
use opentalk_controller_settings::Settings;
use opentalk_db_storage::users::User;
use opentalk_types_api_v1::users::{PrivateUserProfile, PublicUserProfile};

/// A trait providing conversion of database users to public and private user profiles
pub trait ToUserProfile {
    /// Convert to the public user profile
    fn to_public_user_profile(&self, settings: &Settings) -> PublicUserProfile;

    /// Convert to the private user profile
    fn to_private_user_profile(&self, settings: &Settings, used_storage: u64)
        -> PrivateUserProfile;
}

impl ToUserProfile for User {
    fn to_public_user_profile(&self, settings: &Settings) -> PublicUserProfile {
        let default_avatar = email_to_libravatar_url(&settings.avatar.libravatar_url, &self.email);

        PublicUserProfile {
            id: self.id,
            email: self.email.clone(),
            title: self.title.clone(),
            firstname: self.firstname.clone(),
            lastname: self.lastname.clone(),
            display_name: self.display_name.clone(),
            avatar_url: self.avatar_url.clone().unwrap_or(default_avatar),
        }
    }

    fn to_private_user_profile(
        &self,
        settings: &Settings,
        used_storage: u64,
    ) -> PrivateUserProfile {
        let default_avatar = email_to_libravatar_url(&settings.avatar.libravatar_url, &self.email);

        PrivateUserProfile {
            id: self.id,
            email: self.email.clone(),
            title: self.title.clone(),
            firstname: self.firstname.clone(),
            lastname: self.lastname.clone(),
            display_name: self.display_name.clone(),
            dashboard_theme: self.dashboard_theme.clone(),
            conference_theme: self.conference_theme.clone(),
            avatar_url: self.avatar_url.clone().unwrap_or(default_avatar),
            language: self.language.clone(),
            tariff_status: self.tariff_status,
            used_storage,
        }
    }
}

/// Helper function to turn an email address into libravatar URL.
pub fn email_to_libravatar_url(libravatar_url: &str, email: &str) -> String {
    format!("{}{:x}", libravatar_url, md5::compute(email))
}
