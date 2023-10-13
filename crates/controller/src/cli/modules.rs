// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use async_trait::async_trait;
use clap::Subcommand;
use signaling_core::{ModulesRegistrar, RegisterModules, SignalingModule};

#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum Command {
    /// List available modules and their features
    List,
}

struct ModuleConsolePrinter;

#[async_trait(?Send)]
impl ModulesRegistrar for ModuleConsolePrinter {
    async fn register<M: SignalingModule>(&mut self) -> Result<()> {
        println!("{}: {:?}", M::NAMESPACE, M::get_provided_features());
        Ok(())
    }
}

pub async fn handle_command<M: RegisterModules>(command: Command) -> Result<()> {
    match command {
        Command::List => M::register(&mut ModuleConsolePrinter).await,
    }
}
