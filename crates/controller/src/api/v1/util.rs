// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::response::ApiError;
use controller_settings::Settings;
use database::DbConnection;
use database::Result;
use db_storage::tariffs::Tariff;
use db_storage::users::User;
use db_storage::utils::HasUsers;
use std::collections::HashMap;
use types::api::v1::users::PublicUserProfile;
use types::core::UserId;

/// Utility to fetch user profiles batched
///
/// See [`db_storage::utils::HasUsers`]
#[derive(Default)]
pub struct GetUserProfilesBatched {
    users: Vec<UserId>,
}

impl GetUserProfilesBatched {
    pub fn new() -> Self {
        Self { users: vec![] }
    }

    pub fn add(&mut self, has_users: impl HasUsers) -> &mut Self {
        has_users.populate(&mut self.users);
        self
    }

    pub async fn fetch(
        &mut self,
        settings: &Settings,
        conn: &mut DbConnection,
    ) -> Result<UserProfilesBatch> {
        if self.users.is_empty() {
            return Ok(UserProfilesBatch {
                users: HashMap::new(),
            });
        }

        self.users.sort_unstable();
        self.users.dedup();

        User::get_all_by_ids(conn, &self.users)
            .await
            .map(|users| {
                users
                    .into_iter()
                    .map(|user| (user.id, user.to_public_user_profile(settings)))
                    .collect()
            })
            .map(|users| UserProfilesBatch { users })
    }
}

pub struct UserProfilesBatch {
    users: HashMap<UserId, PublicUserProfile>,
}

impl UserProfilesBatch {
    pub fn get(&self, id: UserId) -> PublicUserProfile {
        self.users
            .get(&id)
            .expect("tried to get user-profile without fetching it first")
            .clone()
    }
}

/// Checks if the given feature sting is disabled by the tariff of the given user or in the settings of the controller.
///
/// Return an [`ApiError`] if the given feature is disabled, differentiating between a config disable or tariff restriction.
pub(crate) async fn require_feature(
    db_conn: &mut DbConnection,
    settings: &Settings,
    user_id: UserId,
    feature: &str,
) -> Result<(), ApiError> {
    if settings.defaults.disabled_features().contains(feature) {
        return Err(ApiError::forbidden()
            .with_code("feature_disabled")
            .with_message(format!("The feature \"{feature}\" is disabled")));
    }

    let tariff = Tariff::get_by_user_id(db_conn, &user_id).await?;

    if tariff.is_feature_disabled(feature) {
        return Err(ApiError::forbidden()
            .with_code("feature_disabled_by_tariff")
            .with_message(format!(
                "The user's tariff does not include the {feature} feature"
            )));
    }

    Ok(())
}
