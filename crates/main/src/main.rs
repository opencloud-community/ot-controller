// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use community_modules::CommunityModules;
use controller::Controller;

#[actix_web::main]
async fn main() {
    controller::try_or_exit(run()).await;
}

async fn run() -> Result<()> {
    match std::env::args().next() {
        Some(s) if s.ends_with("k3k-controller") => {
            use owo_colors::OwoColorize as _;
            anstream::eprintln!(
                "{}: It appears you're using the deprecated `k3k-controller` executable, \
                you should be using the `opentalk-controller` executable instead. \
                The `k3k-controller` executable will be removed in a future release.",
                "DEPRECATION WARNING".yellow().bold(),
            );
        }
        _ => {}
    }
    if let Some(controller) =
        Controller::create::<CommunityModules>("OpenTalk Controller Community Edition").await?
    {
        controller.run().await?;
    }

    Ok(())
}
