// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, Result};
use clap::{ArgAction, Parser, Subcommand};
use controller_settings::Settings;
use signaling_core::RegisterModules;

mod acl;
mod fix_acl;
mod jobs;
mod modules;
mod reload;
mod tariffs;
mod tenants;

#[derive(Parser, Debug, Clone)]
#[clap(name = "opentalk-controller")]
pub struct Args {
    #[clap(
        short,
        long,
        default_value = "config.toml",
        help = "Specify path to configuration file"
    )]
    pub config: String,

    /// Triggers a reload of the Janus Server configuration
    #[clap(long)]
    pub reload: bool,

    #[clap(subcommand)]
    cmd: Option<SubCommand>,

    #[clap(long, action=ArgAction::SetTrue)]
    version: bool,
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
        !(self.reload || self.cmd.is_some() || self.version)
    }
}

/// Parses the CLI-Arguments into [`Args`]
///
/// Also runs (optional) cli commands if necessary
pub async fn parse_args<M: RegisterModules>() -> Result<Args> {
    let args = Args::parse();

    if args.version {
        print_version()
    }

    if args.reload {
        reload::trigger_reload()?;
    }
    if let Some(sub_command) = args.cmd.clone() {
        let settings = Settings::load(&args.config)?;
        match sub_command {
            SubCommand::FixAcl(args) => {
                fix_acl::fix_acl(settings, args).await?;
            }
            SubCommand::Acl(subcommand) => {
                acl::acl(settings, subcommand).await?;
            }
            SubCommand::MigrateDb => {
                let result = db_storage::migrations::migrate_from_url(&settings.database.url)
                    .await
                    .context("Failed to migrate database")?;
                println!("{result:?}");
            }
            SubCommand::Tenants(command) => {
                tenants::handle_command(settings, command).await?;
            }
            SubCommand::Tariffs(command) => {
                tariffs::handle_command(settings, command).await?;
            }
            SubCommand::Jobs(command) => {
                jobs::handle_command(settings, command).await?;
            }
            SubCommand::Modules(command) => {
                modules::handle_command::<M>(command).await?;
            }
        }
    }

    Ok(args)
}

const BUILD_INFO: [(&str, Option<&str>); 10] = [
    ("Build Timestamp", option_env!("VERGEN_BUILD_TIMESTAMP")),
    ("Build Version", option_env!("VERGEN_BUILD_SEMVER")),
    ("Commit SHA", option_env!("VERGEN_GIT_SHA")),
    ("Commit Date", option_env!("VERGEN_GIT_COMMIT_TIMESTAMP")),
    ("Commit Branch", option_env!("VERGEN_GIT_BRANCH")),
    ("rustc Version", option_env!("VERGEN_RUSTC_SEMVER")),
    ("rustc Channel", option_env!("VERGEN_RUSTC_CHANNEL")),
    ("rustc Host Triple", option_env!("VERGEN_RUSTC_HOST_TRIPLE")),
    (
        "cargo Target Triple",
        option_env!("VERGEN_CARGO_TARGET_TRIPLE"),
    ),
    ("cargo Profile", option_env!("VERGEN_CARGO_PROFILE")),
];

fn print_version() {
    for (label, value) in BUILD_INFO {
        println!("{}: {}", label, value.unwrap_or("N/A"));
    }
}
