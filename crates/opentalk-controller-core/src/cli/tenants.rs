// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::Utc;
use clap::Subcommand;
use opentalk_controller_settings::Settings;
use opentalk_database::{DatabaseError, Db};
use opentalk_db_storage::tenants::{OidcTenantId, Tenant, UpdateTenant};
use opentalk_types_common::tenants::TenantId;
use tabled::{settings::Style, Table, Tabled};
use uuid::Uuid;

#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum Command {
    /// List all available tenants
    List,
    /// Change a tenants oidc-id
    SetOidcId { id: Uuid, new_oidc_id: String },
}

pub async fn handle_command(settings: &Settings, command: Command) -> Result<(), DatabaseError> {
    match command {
        Command::List => list_all_tenants(settings).await,
        Command::SetOidcId { id, new_oidc_id } => {
            set_oidc_id(
                settings,
                TenantId::from(id),
                OidcTenantId::from(new_oidc_id),
            )
            .await
        }
    }
}

#[derive(Tabled)]
struct TenantTableRow {
    id: TenantId,
    oidc_id: OidcTenantId,
}

impl TenantTableRow {
    fn from_tenant(tenant: Tenant) -> Self {
        Self {
            id: tenant.id,
            oidc_id: tenant.oidc_tenant_id,
        }
    }
}

/// Implementation of the `opentalk-controller tenants list` command
async fn list_all_tenants(settings: &Settings) -> Result<(), DatabaseError> {
    let db = Db::connect(&settings.raw.database)?;
    let mut conn = db.get_conn().await?;

    let tenants = Tenant::get_all(&mut conn).await?;
    let rows: Vec<TenantTableRow> = tenants
        .into_iter()
        .map(TenantTableRow::from_tenant)
        .collect();

    println!("{}", Table::new(rows).with(Style::psql()));

    Ok(())
}

/// Implementation of the `opentalk-controller tenants set-oidc-id <tenant-id> <new-oidc-id>` command
async fn set_oidc_id(
    settings: &Settings,
    id: TenantId,
    new_oidc_id: OidcTenantId,
) -> Result<(), DatabaseError> {
    let db = Db::connect(&settings.raw.database)?;
    let mut conn = db.get_conn().await?;

    let tenant = Tenant::get(&mut conn, id).await?;
    let old_oidc_id = tenant.oidc_tenant_id;

    UpdateTenant {
        updated_at: Utc::now(),
        oidc_tenant_id: &new_oidc_id,
    }
    .apply(&mut conn, id)
    .await?;

    println!(
        "Updated tenant's oidc-id\n\tid  = {id}\n\told = {old_oidc_id}\n\tnew = {new_oidc_id}"
    );

    Ok(())
}
