// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use clap::Subcommand;
use signaling_core::{ModulesRegistrar, RegisterModules, SignalingModule};

#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum Command {
    /// List available modules and their features
    List,
}

struct ModuleConsolePrinter;

impl ModulesRegistrar for ModuleConsolePrinter {
    fn register<M: SignalingModule>(&mut self) {
        println!("{}: {:?}", M::NAMESPACE, M::get_provided_features());
    }
}

pub async fn handle_command<M: RegisterModules>(command: Command) -> Result<()> {
    match command {
        Command::List => {
            M::register(&mut ModuleConsolePrinter);
            Ok(())
        }
    }
}
