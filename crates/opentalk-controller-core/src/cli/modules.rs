// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::convert::Infallible;

use async_trait::async_trait;
use clap::Subcommand;
use itertools::Itertools as _;
use opentalk_signaling_core::{ModulesRegistrar, RegisterModules, SignalingModule};

#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum Command {
    /// List available modules and their features
    List,
}

struct ModuleConsolePrinter;

#[async_trait(?Send)]
impl ModulesRegistrar for ModuleConsolePrinter {
    type Error = Infallible;

    async fn register<M: SignalingModule>(&mut self) -> Result<(), Infallible> {
        println!(
            "{}: [{}]",
            M::NAMESPACE,
            M::get_provided_features()
                .into_iter()
                .map(|f| format!("\"{f}\""))
                .join(", ")
        );
        Ok(())
    }
}

pub async fn handle_command<M: RegisterModules>(command: Command) -> Result<(), Infallible> {
    match command {
        Command::List => M::register(&mut ModuleConsolePrinter).await,
    }
}
