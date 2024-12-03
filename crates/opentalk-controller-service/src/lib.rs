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

use std::sync::Arc;

use async_trait::async_trait;
use opentalk_controller_service_facade::OpenTalkControllerServiceBackend;
use opentalk_controller_settings::{Settings, SharedSettings};
use opentalk_database::Db;
use opentalk_db_storage::{events::Event, rooms::Room, users::User, utils::build_event_info};
use opentalk_types::api::error::ApiError;
use opentalk_types_api_v1::{
    auth::{GetLoginResponseBody, OidcProvider},
    rooms::{by_room_id::GetRoomEventResponseBody, RoomResource},
    users::{PrivateUserProfile, PublicUserProfile},
};
use opentalk_types_common::rooms::RoomId;

/// The default [`OpenTalkControllerServiceBackend`] implementation.
pub struct ControllerBackend {
    db: Arc<Db>,
    // TODO: these are ArcSwap in controller-core, investigate what exactly that provides and what it is used for
    settings: SharedSettings,
    frontend_oidc_provider: OidcProvider,
}

impl ControllerBackend {
    /// Create a new [`ControllerBackend`].
    pub fn new(
        settings: SharedSettings,
        db: Arc<Db>,
        frontend_oidc_provider: OidcProvider,
    ) -> Self {
        Self {
            settings,
            db,
            frontend_oidc_provider,
        }
    }
}

impl std::fmt::Debug for ControllerBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ControllerBackend")
    }
}

#[async_trait]
impl OpenTalkControllerServiceBackend for ControllerBackend {
    async fn get_login(&self) -> GetLoginResponseBody {
        GetLoginResponseBody {
            oidc: self.frontend_oidc_provider.clone(),
        }
    }

    async fn get_room(&self, room_id: &RoomId) -> Result<RoomResource, ApiError> {
        let settings = self.settings.load();

        let mut conn = self.db.get_conn().await?;

        let (room, created_by) = Room::get_with_user(&mut conn, *room_id).await?;

        let room_resource = RoomResource {
            id: room.id,
            created_by: created_by.to_public_user_profile(&settings),
            created_at: room.created_at.into(),
            password: room.password,
            waiting_room: room.waiting_room,
        };

        Ok(room_resource)
    }

    async fn get_room_event(&self, room_id: &RoomId) -> Result<GetRoomEventResponseBody, ApiError> {
        let settings = self.settings.load();

        let mut conn = self.db.get_conn().await?;

        let event = Event::get_for_room(&mut conn, *room_id).await?;

        let room = Room::get(&mut conn, *room_id).await?;

        match event.as_ref() {
            Some(event) => {
                let call_in_tel = settings.call_in.as_ref().map(|call_in| call_in.tel.clone());
                let event_info =
                    build_event_info(&mut conn, call_in_tel, *room_id, room.e2e_encryption, event)
                        .await?;
                Ok(GetRoomEventResponseBody(event_info))
            }
            None => Err(ApiError::not_found()),
        }
    }
}

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
