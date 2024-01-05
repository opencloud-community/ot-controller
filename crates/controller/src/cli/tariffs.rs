// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, Result};
use chrono::Utc;
use clap::Subcommand;
use controller_settings::Settings;
use database::{Db, DbConnection};
use db_storage::tariffs::{ExternalTariff, ExternalTariffId, NewTariff, Tariff, UpdateTariff};
use db_storage::utils::Jsonb;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::AsyncConnection;
use itertools::Itertools;
use std::collections::HashMap;
use tabled::{settings::Style, Table, Tabled};

#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum Command {
    /// List all available tariffs
    List,
    /// Create a new tariff
    Create {
        /// Name of the tariff
        tariff_name: String,
        /// Eternal ID to map to the tariff
        external_tariff_id: String,
        /// Comma-separated list of modules to disable
        #[clap(long, value_delimiter = ',')]
        disabled_modules: Vec<String>,
        /// Comma-separated list of features to disable
        #[clap(long, value_delimiter = ',')]
        disabled_features: Vec<String>,
        /// Comma-separated list of key=value pairs
        #[clap(long, value_delimiter = ',', value_parser = parse_quota)]
        quotas: Vec<(String, u32)>,
    },
    /// Delete a tariff by name
    Delete {
        /// Name of the tariff to delete
        tariff_name: String,
    },

    /// Modify an existing tariff
    Edit {
        /// Name of the tariff to modify
        tariff_name: String,

        /// Set a new name
        #[clap(long)]
        set_name: Option<String>,

        /// Comma-separated list of external tariff_ids to add
        #[clap(long, value_delimiter = ',')]
        add_external_tariff_ids: Vec<String>,

        /// Comma-separated list of external tariff_ids to remove
        #[clap(long, value_delimiter = ',')]
        remove_external_tariff_ids: Vec<String>,

        /// Comma-separated list of module names to add
        #[clap(long, value_delimiter = ',')]
        add_disabled_modules: Vec<String>,

        /// Comma-separated list of module names to remove
        #[clap(long, value_delimiter = ',')]
        remove_disabled_modules: Vec<String>,

        /// Comma-separated list of feature names to add
        #[clap(long, value_delimiter = ',')]
        add_disabled_features: Vec<String>,

        /// Comma-separated list of feature names to remove
        #[clap(long, value_delimiter = ',')]
        remove_disabled_features: Vec<String>,

        /// Comma-separated list of key=value pairs to add, overwrites quotas with the same name
        #[clap(long, value_delimiter = ',', value_parser = parse_quota)]
        add_quotas: Vec<(String, u32)>,

        /// Comma-separated list of quota keys to remove
        #[clap(long, value_delimiter = ',')]
        remove_quotas: Vec<String>,
    },
}

fn parse_quota(s: &str) -> Result<(String, u32)> {
    let (name, value) = s.split_once('=').context("invalid key=value pair")?;
    let value = value
        .trim()
        .parse()
        .context("invalid quota value, expected u32")?;
    Ok((name.trim().into(), value))
}

pub async fn handle_command(settings: Settings, command: Command) -> Result<()> {
    match command {
        Command::List => list_all_tariffs(settings).await,
        Command::Create {
            tariff_name,
            external_tariff_id,
            disabled_modules,
            disabled_features,
            quotas,
        } => {
            create_tariff(
                settings,
                tariff_name,
                external_tariff_id,
                disabled_modules,
                disabled_features,
                quotas.into_iter().collect(),
            )
            .await
        }
        Command::Delete { tariff_name } => delete_tariff(settings, tariff_name).await,
        Command::Edit {
            tariff_name,
            set_name,
            add_external_tariff_ids,
            remove_external_tariff_ids,
            add_disabled_modules,
            remove_disabled_modules,
            add_disabled_features,
            remove_disabled_features,
            add_quotas,
            remove_quotas,
        } => {
            edit_tariff(
                settings,
                tariff_name,
                set_name,
                add_external_tariff_ids,
                remove_external_tariff_ids,
                add_disabled_modules,
                remove_disabled_modules,
                add_disabled_features,
                remove_disabled_features,
                add_quotas.into_iter().collect(),
                remove_quotas,
            )
            .await
        }
    }
}

async fn list_all_tariffs(settings: Settings) -> Result<()> {
    let db = Db::connect(&settings.database).context("Failed to connect to database")?;
    let mut conn = db.get_conn().await?;

    let tariffs = Tariff::get_all(&mut conn).await?;

    print_tariffs(&mut conn, tariffs).await
}

async fn create_tariff(
    settings: Settings,
    name: String,
    external_tariff_id: String,
    disabled_modules: Vec<String>,
    disabled_features: Vec<String>,
    quotas: HashMap<String, u32>,
) -> Result<()> {
    let db = Db::connect(&settings.database).context("Failed to connect to database")?;
    let mut conn = db.get_conn().await?;

    conn.transaction(|conn| async move {
        let tariff = NewTariff {
            name: name.clone(),
            quotas: Jsonb(quotas),
            disabled_modules,
            disabled_features,
        }
        .insert(conn).await?;

        ExternalTariff {
            external_id: ExternalTariffId::from(external_tariff_id.clone()),
            tariff_id: tariff.id,
        }
        .insert(conn).await?;

        println!(
            "Created tariff name={name:?} with external external_tariff_id={external_tariff_id:?} ({})",
            tariff.id
        );

        Ok(())
    }
    .scope_boxed()).await
}

async fn delete_tariff(settings: Settings, name: String) -> Result<()> {
    let db = Db::connect(&settings.database).context("Failed to connect to database")?;
    let mut conn = db.get_conn().await?;

    conn.transaction(|conn| {
        async move {
            let tariff = Tariff::get_by_name(conn, &name).await?;
            ExternalTariff::delete_all_for_tariff(conn, tariff.id).await?;
            Tariff::delete_by_id(conn, tariff.id).await?;

            println!("Deleted tariff name={name:?} ({})", tariff.id);

            Ok(())
        }
        .scope_boxed()
    })
    .await
}

#[allow(clippy::too_many_arguments)]
async fn edit_tariff(
    settings: Settings,
    name: String,
    set_name: Option<String>,
    add_external_tariff_ids: Vec<String>,
    remove_external_tariff_ids: Vec<String>,
    add_disabled_modules: Vec<String>,
    remove_disabled_modules: Vec<String>,
    add_disabled_features: Vec<String>,
    remove_disabled_features: Vec<String>,
    add_quotas: HashMap<String, u32>,
    remove_quotas: Vec<String>,
) -> Result<()> {
    let db = Db::connect(&settings.database).context("Failed to connect to database")?;
    let mut conn = db.get_conn().await?;

    conn.transaction(|conn| {
        async move {
            let tariff = Tariff::get_by_name(conn, &name).await?;

            // Remove all specified external tariff ids
            if !remove_external_tariff_ids.is_empty() {
                let external_tariff_ids_to_remove: Vec<ExternalTariffId> =
                    remove_external_tariff_ids
                        .into_iter()
                        .map(ExternalTariffId::from)
                        .collect();
                ExternalTariff::delete_all_for_tariff_by_external_id(
                    conn,
                    tariff.id,
                    &external_tariff_ids_to_remove,
                )
                .await?;
            }

            // Add all specified external tariff ids
            if !add_external_tariff_ids.is_empty() {
                for to_add in add_external_tariff_ids {
                    ExternalTariff {
                        external_id: ExternalTariffId::from(to_add.clone()),
                        tariff_id: tariff.id,
                    }
                    .insert(conn)
                    .await
                    .with_context(|| format!("Failed to add external tariff_id {to_add:?}"))?;
                }
            }

            // Modify the `disabled_modules` list
            let mut disabled_modules = tariff.disabled_modules();
            disabled_modules
                .retain(|disabled_module| !remove_disabled_modules.contains(disabled_module));
            disabled_modules.extend(add_disabled_modules.into_iter());

            // Modify the `disabled_features` list
            let mut disabled_features = tariff.disabled_features();
            disabled_features
                .retain(|disabled_module| !remove_disabled_features.contains(disabled_module));
            disabled_features.extend(add_disabled_features);

            // Modify the `quotas` set
            let mut quotas = tariff.quotas.0;
            quotas.retain(|key, _| !remove_quotas.contains(key));
            quotas.extend(add_quotas);

            // Apply changeset
            let tariff = UpdateTariff {
                name: set_name,
                updated_at: Utc::now(),
                quotas: Some(Jsonb(quotas)),
                disabled_modules: Some(Vec::from_iter(disabled_modules)),
                disabled_features: Some(Vec::from_iter(disabled_features)),
            }
            .apply(conn, tariff.id)
            .await?;

            println!("Updated tariff name={:?} ({})", tariff.name, tariff.id);
            print_tariffs(conn, [tariff]).await
        }
        .scope_boxed()
    })
    .await
}

/// Print the list of tariffs as table
async fn print_tariffs(
    conn: &mut DbConnection,
    tariffs: impl IntoIterator<Item = Tariff>,
) -> Result<()> {
    #[derive(Tabled)]
    struct TariffTableRow {
        #[tabled(rename = "name (internal)")]
        name: String,
        #[tabled(rename = "external tariff_id")]
        ext: String,
        #[tabled(rename = "disabled modules")]
        disabled_modules: String,
        #[tabled(rename = "disabled features")]
        disabled_features: String,
        quotas: String,
    }

    let mut rows = vec![];

    for tariff in tariffs {
        let ids = ExternalTariff::get_all_for_tariff(conn, tariff.id).await?;
        let mut ids = ids
            .into_iter()
            .map(|ext_tariff_id| ext_tariff_id.to_string())
            .join("\n");
        if ids.is_empty() {
            ids = "-".into();
        }

        let mut disabled_modules = tariff.disabled_modules().into_iter().join("\n");
        if disabled_modules.is_empty() {
            disabled_modules = "-".into();
        }

        let mut disabled_features = tariff.disabled_features().into_iter().join("\n");
        if disabled_features.is_empty() {
            disabled_features = "-".into();
        }

        let mut quotas = tariff
            .quotas
            .0
            .into_iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .join("\n");
        if quotas.is_empty() {
            quotas = "-".into();
        }

        rows.push(TariffTableRow {
            name: tariff.name,
            ext: ids,
            disabled_modules,
            disabled_features,
            quotas,
        });
    }

    println!("{}", Table::new(rows).with(Style::ascii()));

    Ok(())
}
