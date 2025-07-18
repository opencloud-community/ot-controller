// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::path::PathBuf;

use build_info::BuildInfo;
use clap::{Parser, Subcommand};
use opentalk_controller_settings::SettingsProvider;
use opentalk_signaling_core::RegisterModules;
use opentalk_version::InfoArgs;
use snafu::ResultExt;

use crate::Result;

mod acl;
mod fix_acl;
mod jobs;
mod license;
mod modules;
mod openapi;
mod reload;
mod tariffs;
mod tenants;

#[derive(Parser, Debug, Clone)]
#[clap(name = "opentalk-controller")]
pub struct Args {
    /// Path of the configuration file.
    ///
    /// If present, exactly this config file will be used.
    ///
    /// If absent, `controller` looks for a config file in these locations and uses the first one that is found:
    ///
    /// - `config.toml` in the current directory (deprecated, for backwards compatiblity only)
    /// - `controller.toml` in the current directory
    /// - `<XDG_CONFIG_HOME>/opentalk/controller.toml` (where `XDG_CONFIG_HOME` is usually `~/.config`)
    /// - `/etc/opentalk/controller.toml`
    #[clap(short, long, verbatim_doc_comment)]
    pub config: Option<PathBuf>,

    /// Triggers a reload of reloadable configuration options
    #[clap(long)]
    pub reload: bool,

    #[clap(subcommand)]
    cmd: Option<SubCommand>,

    #[command(flatten)]
    pub(crate) info: InfoArgs,
}

#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
#[allow(clippy::large_enum_variant)]
enum SubCommand {
    /// Recreate all ACL entries from the current database content. Existing entries will not be touched unless the
    /// command is told to delete them all beforehand.
    FixAcl(fix_acl::Args),

    /// Modify the ACLs.
    #[clap(subcommand)]
    Acl(AclSubCommand),

    /// Migrate the db. This is done automatically during start of the controller,
    /// but can be done without starting the controller using this command.
    MigrateDb,

    /// Manage existing tenants
    #[clap(subcommand)]
    Tenants(tenants::Command),

    /// Manage tariffs
    #[clap(subcommand)]
    Tariffs(tariffs::Command),

    /// Manage and execute maintenance jobs
    #[clap(subcommand)]
    Jobs(jobs::Command),

    /// Manage modules
    #[clap(subcommand)]
    Modules(modules::Command),

    /// Get information on the OpenAPI specification
    #[clap(subcommand)]
    Openapi(openapi::Command),
}

#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub(crate) enum AclSubCommand {
    /// Allows all users access to all rooms
    UsersHaveAccessToAllRooms {
        /// Enable/Disable
        #[clap(subcommand)]
        action: EnableDisable,
    },
}

#[derive(Parser, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub(crate) enum EnableDisable {
    /// enable
    Enable,
    /// disable
    Disable,
}

impl Args {
    /// Returns true if we want to startup the controller after we finished the cli part
    pub fn controller_should_start(&self) -> bool {
        !(self.reload || self.cmd.is_some() || self.info.should_print())
    }
}

/// Parses the CLI-Arguments into [`Args`]
///
/// Also runs (optional) cli commands if necessary
pub async fn parse_args<M: RegisterModules>() -> Result<Args> {
    let args = Args::parse();

    if args.info.should_print() {
        print_info(&args.info);
    }

    if args.reload {
        reload::trigger_reload()?;
    }
    if let Some(sub_command) = args.cmd.clone() {
        let settings_provider =
            SettingsProvider::load_from_path_or_standard_paths(args.config.as_deref())
                .whatever_context("Failed to load settings")?;
        let settings = settings_provider.get();

        match sub_command {
            SubCommand::FixAcl(args) => {
                fix_acl::fix_acl(&settings, args).await?;
            }
            SubCommand::Acl(subcommand) => {
                acl::acl(&settings, subcommand).await?;
            }
            SubCommand::MigrateDb => {
                let result =
                    opentalk_db_storage::migrations::migrate_from_url(&settings.database.url)
                        .await
                        .whatever_context("Failed to migrate database")?;
                println!("{result:?}");
            }
            SubCommand::Tenants(command) => {
                tenants::handle_command(&settings, command)
                    .await
                    .whatever_context("Tenants command failed")?;
            }
            SubCommand::Tariffs(command) => {
                tariffs::handle_command(&settings, command)
                    .await
                    .whatever_context("Tariffs command failed")?;
            }
            SubCommand::Jobs(command) => {
                jobs::handle_command(&settings, command)
                    .await
                    .whatever_context("Jobs command failed")?;
            }
            SubCommand::Modules(command) => {
                modules::handle_command::<M>(command)
                    .await
                    .whatever_context("Modules command failed")?;
            }
            SubCommand::Openapi(command) => {
                openapi::handle_command(command)
                    .await
                    .whatever_context("OpenAPI command failed")?;
            }
        }
    }

    Ok(args)
}

opentalk_version::build_info!();

fn print_info(info_args: &InfoArgs) {
    let build_info = BuildInfo::with_license(license::LICENSE.to_owned());
    if let Some(text) = build_info.format(info_args) {
        println!("{text}");
    }
}
