// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Fixes acl rules based on the database content

use crate::{
    acl::check_or_create_kustos_default_permissions,
    api::v1::{events::EventPoliciesBuilderExt, rooms::RoomsPoliciesBuilderExt},
};
use anyhow::{bail, Context, Error, Result};
use chrono::Utc;
use clap::Parser;
use controller_settings::Settings;
use database::{Db, DbConnection};
use db_storage::{
    events::Event, invites::Invite, module_resources::ModuleResource, rooms::Room, users::User,
};
use kustos::prelude::*;
use std::sync::Arc;

#[derive(Debug, Clone, Parser)]
pub(super) struct Args {
    /// !DANGER! Removes all ACL entries before running any fixes.
    ///
    /// Requires all fixes to be run.
    #[clap(long, default_value = "false")]
    delete_acl_entries: bool,

    /// Skip user role fix
    #[clap(long, default_value = "false")]
    skip_users: bool,

    /// Skip group membership fix
    #[clap(long, default_value = "false")]
    skip_groups: bool,

    /// Skip fix of room permissions
    #[clap(long, default_value = "false")]
    skip_rooms: bool,

    /// Skip fix of module resources permissions
    #[clap(long, default_value = "false")]
    skip_module_resources: bool,

    /// Skip fix of event permission fixes
    #[clap(long, default_value = "false")]
    skip_events: bool,
}

pub(super) async fn fix_acl(settings: Settings, args: Args) -> Result<()> {
    let db = Arc::new(Db::connect(&settings.database).context("Failed to connect to database")?);
    let mut conn = db
        .get_conn()
        .await
        .context("Failed to get connection from connection pool")?;

    let authz = kustos::Authz::new(db.clone()).await?;

    match &args {
        Args {
            delete_acl_entries: true,
            skip_users: false,
            skip_groups: false,
            skip_rooms: false,
            skip_module_resources: false,
            skip_events: false,
        } => {
            // Only remove all policies if none of the skips are specified
            authz.clear_all_policies().await?;
        }
        Args {
            delete_acl_entries: true,
            ..
        } => {
            bail!("Refusing to delete acl entries if any of the subsequent checks are skipped");
        }
        _ => {}
    }

    check_or_create_kustos_default_permissions(&authz).await?;

    // Used to collect errors during looped operations
    let mut errors: Vec<Error> = Vec::new();

    if !(args.skip_users && args.skip_groups) {
        fix_user(&args, &mut conn, &authz, &mut errors).await?;
    }

    if !args.skip_rooms {
        fix_rooms(&mut conn, &authz).await?;
    }

    if !args.skip_module_resources {
        fix_module_resources(&mut conn, &authz).await?;
    }

    if !args.skip_events {
        fix_events(&mut conn, &authz).await?;
    }

    if errors.is_empty() {
        println!("ACLs fixed");
        Ok(())
    } else {
        use std::fmt::Write;
        bail!(
            "{}",
            errors.iter().fold(String::new(), |mut out, e| {
                let _ = writeln!(out, "{e:#} ");
                out
            })
        )
    }
}

async fn fix_user(
    args: &Args,
    conn: &mut DbConnection,
    authz: &kustos::Authz,
    errors: &mut Vec<Error>,
) -> Result<()> {
    let users = User::get_all_with_groups(conn)
        .await
        .context("Failed to load users")?;

    for (user, groups) in users {
        if !args.skip_users {
            let needs_addition = !match authz.is_user_in_role(user.id, "user").await {
                Ok(in_role) => in_role,
                Err(e) => {
                    errors.push(e.into());
                    false
                }
            };

            if needs_addition {
                match authz.add_user_to_role(user.id, "user").await {
                    Ok(_) => {}
                    Err(e) => errors.push(e.into()),
                }
            }
        }

        if !args.skip_groups {
            for group in groups {
                let needs_addition = !match authz
                    .is_user_in_group(user.id, group.id)
                    .await
                    .with_context(|| format!("User: {}, Group: {}", user.id, group.id))
                {
                    Ok(in_group) => in_group,
                    Err(e) => {
                        errors.push(e);
                        false
                    }
                };

                if needs_addition {
                    match authz
                        .add_user_to_group(user.id, group.id)
                        .await
                        .with_context(|| format!("User: {}, Group: {}", user.id, group.id))
                    {
                        Ok(_) => {}
                        Err(e) => errors.push(e),
                    }
                }
            }
        }
    }
    Ok(())
}

async fn fix_rooms(conn: &mut DbConnection, authz: &kustos::Authz) -> Result<()> {
    let mut policies = PoliciesBuilder::new();

    let rooms = Room::get_all_with_creator(conn)
        .await
        .context("failed to load rooms")?;
    for (room, user) in rooms {
        policies = policies
            .grant_user_access(user.id)
            .room_read_access(room.id)
            .room_write_access(room.id)
            .finish();
    }

    let now = Utc::now();
    let invites = Invite::get_all(conn)
        .await
        .context("failed to load invites")?;
    for Invite {
        id,
        room,
        active,
        expiration,
        ..
    } in invites
    {
        if active && (expiration.is_none() || Some(now) <= expiration) {
            policies = policies
                .grant_invite_access(id)
                .room_guest_read_access(room)
                .finish();
        }
    }

    authz.add_policies(policies).await?;

    Ok(())
}

async fn fix_module_resources(conn: &mut DbConnection, authz: &kustos::Authz) -> Result<()> {
    let module_resources_with_creator = ModuleResource::get_all_with_creator_and_owner(conn)
        .await
        .context("failed to load module resources")?;

    let mut policies = PoliciesBuilder::new();

    for (module_resource_id, creator_id, owner_id) in module_resources_with_creator {
        policies = policies
            .grant_user_access(creator_id)
            .add_resource(
                module_resource_id.resource_id(),
                [AccessMethod::Get, AccessMethod::Put, AccessMethod::Delete],
            )
            .finish();

        policies = policies
            .grant_user_access(owner_id)
            .add_resource(
                module_resource_id.resource_id(),
                [AccessMethod::Get, AccessMethod::Put, AccessMethod::Delete],
            )
            .finish();
    }

    authz.add_policies(policies).await?;

    Ok(())
}

async fn fix_events(conn: &mut DbConnection, authz: &kustos::Authz) -> Result<()> {
    let events_with_creator = Event::get_all_with_creator(conn)
        .await
        .context("failed to load events")?;
    let events_with_invitee = Event::get_all_with_invitee(conn)
        .await
        .context("failed to load events")?;

    let mut policies = PoliciesBuilder::new();

    for (event_id, creator_id) in events_with_creator {
        policies = policies
            .grant_user_access(creator_id)
            .event_read_access(event_id)
            .event_write_access(event_id)
            .finish();
    }

    for (event_id, room_id, creator_id) in events_with_invitee {
        policies = policies
            .grant_user_access(creator_id)
            .room_read_access(room_id)
            .event_read_access(event_id)
            .event_invite_invitee_access(event_id)
            .finish();
    }

    authz.add_policies(policies).await?;

    Ok(())
}
