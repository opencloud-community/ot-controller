// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Allows to manipulate the acls
//! Currently supported is enabling/disabling room access for all users.
use std::sync::Arc;

use kustos::prelude::AccessMethod;
use opentalk_controller_settings::Settings;
use opentalk_database::Db;
use snafu::ResultExt;

use super::AclSubCommand;
use crate::{
    acl::{check_or_create_kustos_role_policy, maybe_remove_kustos_role_policy},
    Result,
};

pub(crate) async fn acl(settings: Settings, e: AclSubCommand) -> Result<()> {
    match e {
        AclSubCommand::UsersHaveAccessToAllRooms { action } => match action {
            super::EnableDisable::Enable => enable_user_access_to_all_rooms(&settings).await?,
            super::EnableDisable::Disable => disable_user_access_to_all_rooms(&settings).await?,
        },
    }
    Ok(())
}

async fn enable_user_access_to_all_rooms(settings: &Settings) -> Result<()> {
    let db = Arc::new(
        Db::connect(&settings.database).whatever_context("Failed to connect to database")?,
    );
    let authz = kustos::Authz::new(db.clone())
        .await
        .whatever_context("Failed to initialize kustos/authz")?;

    check_or_create_kustos_role_policy(&authz, "user", "/rooms/*/start", AccessMethod::POST)
        .await?;
    check_or_create_kustos_role_policy(&authz, "user", "/rooms/*", AccessMethod::GET).await?;
    println!("Enabled access for all users to all rooms");
    Ok(())
}

async fn disable_user_access_to_all_rooms(settings: &Settings) -> Result<()> {
    let db = Arc::new(
        Db::connect(&settings.database).whatever_context("Failed to connect to database")?,
    );
    let authz = kustos::Authz::new(db.clone())
        .await
        .whatever_context("Failed to initialize kustos/authz")?;

    maybe_remove_kustos_role_policy(&authz, "user", "/rooms/*/start", AccessMethod::POST).await?;
    maybe_remove_kustos_role_policy(&authz, "user", "/rooms/*", AccessMethod::GET).await?;
    println!("Disabled access for all users to all rooms");
    Ok(())
}
