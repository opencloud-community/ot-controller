// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Provides some helper functions and the like.

use opentalk_controller_service_facade::RequestUser;
use opentalk_controller_settings::Settings;
use opentalk_controller_utils::CaptureApiError;
use opentalk_database::DbConnection;
use opentalk_db_storage::{assets::Asset, tariffs::Tariff, users::User};
use opentalk_types_api_v1::{
    assets::AssetResource,
    error::ApiError,
    users::{PrivateUserProfile, PublicUserProfile},
};
use opentalk_types_common::{
    features::ModuleFeatureId,
    users::{UserId, UserInfo},
};

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
            user_info: UserInfo {
                title: self.title.clone(),
                firstname: self.firstname.clone(),
                lastname: self.lastname.clone(),
                display_name: self.display_name.clone(),
                avatar_url: self.avatar_url.clone().unwrap_or(default_avatar),
            },
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

impl ToUserProfile for RequestUser {
    fn to_public_user_profile(&self, settings: &Settings) -> PublicUserProfile {
        let default_avatar = email_to_libravatar_url(&settings.avatar.libravatar_url, &self.email);

        PublicUserProfile {
            id: self.id,
            email: self.email.clone(),
            user_info: UserInfo {
                title: self.title.clone(),
                firstname: self.firstname.clone(),
                lastname: self.lastname.clone(),
                display_name: self.display_name.clone(),
                avatar_url: self.avatar_url.clone().unwrap_or(default_avatar),
            },
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

/// Checks if the given feature sting is disabled by the tariff of the given user or in the settings of the controller.
///
/// Return an [`ApiError`] if the given feature is disabled, differentiating between a config disable or tariff restriction.
pub async fn require_feature(
    db_conn: &mut DbConnection,
    settings: &Settings,
    user_id: UserId,
    feature: &ModuleFeatureId,
) -> opentalk_database::Result<(), CaptureApiError> {
    if settings.raw.defaults.disabled_features.contains(feature) {
        return Err(ApiError::forbidden()
            .with_code("feature_disabled")
            .with_message(format!("The feature \"{feature}\" is disabled"))
            .into());
    }

    let tariff = Tariff::get_by_user_id(db_conn, &user_id).await?;

    if tariff.is_feature_disabled(feature) {
        return Err(ApiError::forbidden()
            .with_code("feature_disabled_by_tariff")
            .with_message(format!(
                "The user's tariff does not include the {feature} feature"
            ))
            .into());
    }

    Ok(())
}

/// Converts an Asset from the database to an asset resource
pub fn asset_to_asset_resource(asset: Asset) -> AssetResource {
    let Asset {
        id,
        created_at,
        updated_at: _,
        namespace,
        kind,
        filename,
        tenant_id: _,
        size,
    } = asset;
    AssetResource {
        id,
        filename,
        namespace,
        created_at,
        kind,
        size,
    }
}
